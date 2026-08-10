//! Goal 138: Feature flags.
//!
//! Runtime feature flags for gradual rollouts, A/B testing, and environment-
//! specific features. Flags can be set at build time, runtime, or via a
//! remote config service.

use std::collections::HashMap;

/// A feature flag definition.
#[derive(Debug, Clone)]
pub struct FeatureFlag {
    /// Flag key.
    pub key: String,
    /// Whether the flag is enabled.
    pub enabled: bool,
    /// Description.
    pub description: String,
    /// Rollout percentage (0-100). 100 = fully enabled.
    pub rollout: u8,
    /// Targeted user segments.
    pub segments: Vec<String>,
}

impl FeatureFlag {
    /// Create a new enabled flag.
    pub fn on(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            enabled: true,
            description: String::new(),
            rollout: 100,
            segments: Vec::new(),
        }
    }

    /// Create a new disabled flag.
    pub fn off(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            enabled: false,
            description: String::new(),
            rollout: 0,
            segments: Vec::new(),
        }
    }

    /// Set description.
    pub fn describe(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set rollout percentage.
    pub fn rollout(mut self, percent: u8) -> Self {
        self.rollout = percent.min(100);
        self.enabled = self.rollout > 0;
        self
    }

    /// Target a segment.
    pub fn segment(mut self, segment: impl Into<String>) -> Self {
        self.segments.push(segment.into());
        self
    }

    /// Check if a user should get this flag.
    pub fn is_enabled_for(&self, user_id: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if self.rollout >= 100 {
            return true;
        }
        if self.rollout == 0 {
            return false;
        }
        // Hash user_id to determine if they're in the rollout percentage
        let hash = simple_hash(user_id);
        let bucket = hash % 100;
        bucket < self.rollout as u32
    }
}

fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// Feature flag manager.
pub struct FeatureFlags {
    /// Registered flags.
    flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlags {
    /// Create a new empty flag manager.
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    /// Register a flag.
    pub fn register(&mut self, flag: FeatureFlag) {
        self.flags.insert(flag.key.clone(), flag);
    }

    /// Check if a flag is enabled.
    pub fn is_on(&self, key: &str) -> bool {
        self.flags.get(key).map(|f| f.enabled).unwrap_or(false)
    }

    /// Check if a flag is enabled for a specific user.
    pub fn is_on_for(&self, key: &str, user_id: &str) -> bool {
        self.flags
            .get(key)
            .map(|f| f.is_enabled_for(user_id))
            .unwrap_or(false)
    }

    /// Enable a flag.
    pub fn enable(&mut self, key: &str) {
        if let Some(f) = self.flags.get_mut(key) {
            f.enabled = true;
            f.rollout = 100;
        }
    }

    /// Disable a flag.
    pub fn disable(&mut self, key: &str) {
        if let Some(f) = self.flags.get_mut(key) {
            f.enabled = false;
            f.rollout = 0;
        }
    }

    /// Set rollout percentage for a flag.
    pub fn set_rollout(&mut self, key: &str, percent: u8) {
        if let Some(f) = self.flags.get_mut(key) {
            f.rollout = percent.min(100);
            f.enabled = f.rollout > 0;
        }
    }

    /// Get all registered flags.
    pub fn all(&self) -> Vec<&FeatureFlag> {
        self.flags.values().collect()
    }

    /// Load flags from JSON.
    pub fn from_json(json: &str) -> Self {
        let mut flags = Self::new();
        // Simple parsing — in production, use serde
        // Here we just parse a basic format
        for entry in json.trim_matches(|c| c == '{' || c == '}').split(',') {
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim().trim_matches('"');
                let val = parts[1].trim().trim_matches('"');
                if val == "true" {
                    flags.register(FeatureFlag::on(key));
                } else if val == "false" {
                    flags.register(FeatureFlag::off(key));
                }
            }
        }
        flags
    }

    /// Export flags to JSON.
    pub fn to_json(&self) -> String {
        let entries: Vec<String> = self
            .flags
            .iter()
            .map(|(k, f)| format!(r#""{}":{}"#, k, f.enabled))
            .collect();
        format!("{{{}}}", entries.join(","))
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate the JS for client-side feature flags.
pub fn feature_flags_script(flags: &FeatureFlags) -> String {
    format!(
        r#"<script>
window.__RYE_FLAGS__ = {};
window.__rye_flag = function(key, userId) {{
  var flag = window.__RYE_FLAGS__[key];
  if (flag === undefined) return false;
  if (typeof flag === 'boolean') return flag;
  if (flag.rollout !== undefined && userId) {{
    var hash = 5381;
    for (var i = 0; i < userId.length; i++) {{
      hash = ((hash * 33) + userId.charCodeAt(i)) & 0xffffffff;
    }}
    return (hash % 100) < flag.rollout;
  }}
  return !!flag;
}};
</script>"#,
        flags.to_json()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_on() {
        let flag = FeatureFlag::on("new_ui").describe("New UI redesign");
        assert!(flag.enabled);
        assert_eq!(flag.rollout, 100);
    }

    #[test]
    fn test_feature_flag_off() {
        let flag = FeatureFlag::off("legacy_api");
        assert!(!flag.enabled);
    }

    #[test]
    fn test_feature_flag_rollout() {
        let flag = FeatureFlag::on("beta").rollout(50);
        assert_eq!(flag.rollout, 50);
        assert!(flag.enabled);
    }

    #[test]
    fn test_feature_flag_is_enabled_for_100() {
        let flag = FeatureFlag::on("test").rollout(100);
        assert!(flag.is_enabled_for("user1"));
        assert!(flag.is_enabled_for("user2"));
    }

    #[test]
    fn test_feature_flag_is_enabled_for_0() {
        let flag = FeatureFlag::on("test").rollout(0);
        assert!(!flag.is_enabled_for("user1"));
    }

    #[test]
    fn test_feature_flag_is_enabled_for_50() {
        let flag = FeatureFlag::on("test").rollout(50);
        // With 50% rollout, some users should get it and some shouldn't
        let enabled_count = (0..100)
            .filter(|i| flag.is_enabled_for(&format!("user{}", i)))
            .count();
        assert!(enabled_count > 0 && enabled_count < 100);
    }

    #[test]
    fn test_feature_flags_manager() {
        let mut flags = FeatureFlags::new();
        flags.register(FeatureFlag::on("new_ui"));
        flags.register(FeatureFlag::off("legacy"));

        assert!(flags.is_on("new_ui"));
        assert!(!flags.is_on("legacy"));
        assert!(!flags.is_on("unknown"));
    }

    #[test]
    fn test_feature_flags_enable_disable() {
        let mut flags = FeatureFlags::new();
        flags.register(FeatureFlag::off("test"));
        assert!(!flags.is_on("test"));
        flags.enable("test");
        assert!(flags.is_on("test"));
        flags.disable("test");
        assert!(!flags.is_on("test"));
    }

    #[test]
    fn test_feature_flags_to_json() {
        let mut flags = FeatureFlags::new();
        flags.register(FeatureFlag::on("a"));
        flags.register(FeatureFlag::off("b"));
        let json = flags.to_json();
        assert!(json.contains(r#""a":true"#));
        assert!(json.contains(r#""b":false"#));
    }

    #[test]
    fn test_feature_flags_script() {
        let mut flags = FeatureFlags::new();
        flags.register(FeatureFlag::on("test"));
        let script = feature_flags_script(&flags);
        assert!(script.contains("__RYE_FLAGS__"));
        assert!(script.contains("__rye_flag"));
    }
}
