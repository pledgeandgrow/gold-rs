//! Goal 125: Progressive / partial hydration.
//!
//! Hydrate components as they become visible (intersection observer) or
//! idle (requestIdleCallback). Critical components hydrate first, below-the-fold
//! components hydrate lazily.

/// Hydration priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HydrationPriority {
    /// Hydrate immediately (critical, above-the-fold).
    Immediate = 0,
    /// Hydrate on idle (high priority, but not blocking).
    OnIdle = 1,
    /// Hydrate when visible (intersection observer).
    OnVisible = 2,
    /// Hydrate on user interaction (click, focus).
    OnInteraction = 3,
    /// Hydrate when CPU is idle and all higher priorities are done.
    Lazy = 4,
}

impl HydrationPriority {
    /// Convert to string for the `data-hydrate` attribute.
    pub fn as_str(&self) -> &'static str {
        match self {
            HydrationPriority::Immediate => "immediate",
            HydrationPriority::OnIdle => "idle",
            HydrationPriority::OnVisible => "visible",
            HydrationPriority::OnInteraction => "interaction",
            HydrationPriority::Lazy => "lazy",
        }
    }
}

/// Progressive hydration configuration for a component.
#[derive(Debug, Clone)]
pub struct ProgressiveHydrationConfig {
    /// Hydration priority.
    pub priority: HydrationPriority,
    /// Root margin for intersection observer (if OnVisible).
    pub root_margin: String,
    /// User interaction events to trigger hydration (if OnInteraction).
    pub interaction_events: Vec<String>,
    /// Whether to fallback to immediate if no trigger fires.
    pub fallback_immediate: bool,
    /// Timeout in ms before forcing hydration.
    pub timeout_ms: Option<u32>,
}

impl Default for ProgressiveHydrationConfig {
    fn default() -> Self {
        Self {
            priority: HydrationPriority::OnVisible,
            root_margin: "200px".to_string(),
            interaction_events: vec!["click".to_string(), "focusin".to_string()],
            fallback_immediate: true,
            timeout_ms: Some(5000),
        }
    }
}

impl ProgressiveHydrationConfig {
    /// Create config for immediate hydration.
    pub fn immediate() -> Self {
        Self {
            priority: HydrationPriority::Immediate,
            ..Default::default()
        }
    }

    /// Create config for on-visible hydration.
    pub fn on_visible(root_margin: impl Into<String>) -> Self {
        Self {
            priority: HydrationPriority::OnVisible,
            root_margin: root_margin.into(),
            ..Default::default()
        }
    }

    /// Create config for on-interaction hydration.
    pub fn on_interaction(events: Vec<String>) -> Self {
        Self {
            priority: HydrationPriority::OnInteraction,
            interaction_events: events,
            ..Default::default()
        }
    }

    /// Create config for lazy hydration.
    pub fn lazy() -> Self {
        Self {
            priority: HydrationPriority::Lazy,
            ..Default::default()
        }
    }
}

/// Generate the `data-hydrate` attribute value for a component.
pub fn hydration_attr(config: &ProgressiveHydrationConfig) -> String {
    let mut attr = config.priority.as_str().to_string();
    if config.priority == HydrationPriority::OnVisible {
        attr.push_str(&format!("|rootMargin={}", config.root_margin));
    }
    if config.priority == HydrationPriority::OnInteraction {
        attr.push_str(&format!("|events={}", config.interaction_events.join(",")));
    }
    if let Some(timeout) = config.timeout_ms {
        attr.push_str(&format!("|timeout={}", timeout));
    }
    if config.fallback_immediate {
        attr.push_str("|fallback=immediate");
    }
    attr
}

/// Generate the JS for progressive hydration.
pub fn progressive_hydration_script() -> &'static str {
    r#"<script>
(function() {
  var hydrated = new Set();

  function hydrate(element) {
    var id = element.getAttribute('data-rye-id');
    if (!id || hydrated.has(id)) return;
    hydrated.add(id);

    if (window.__rye_hydrate_component) {
      window.__rye_hydrate_component(element);
    }
  }

  function parseConfig(element) {
    var attr = element.getAttribute('data-hydrate') || 'immediate';
    var parts = attr.split('|');
    var priority = parts[0];
    var config = { priority: priority };
    parts.slice(1).forEach(function(p) {
      var kv = p.split('=');
      config[kv[0]] = kv[1];
    });
    return config;
  }

  // Process all elements with data-hydrate
  function processElements() {
    var elements = document.querySelectorAll('[data-hydrate]');
    elements.forEach(function(el) {
      var config = parseConfig(el);

      if (config.priority === 'immediate') {
        hydrate(el);
      } else if (config.priority === 'idle') {
        if ('requestIdleCallback' in window) {
          requestIdleCallback(function() { hydrate(el); });
        } else {
          setTimeout(function() { hydrate(el); }, 50);
        }
      } else if (config.priority === 'visible') {
        var observer = new IntersectionObserver(function(entries) {
          if (entries[0].isIntersecting) {
            hydrate(el);
            observer.disconnect();
          }
        }, { rootMargin: config.rootMargin || '200px' });
        observer.observe(el);
      } else if (config.priority === 'interaction') {
        var events = (config.events || 'click').split(',');
        function onInteraction() {
          hydrate(el);
          events.forEach(function(e) { el.removeEventListener(e, onInteraction); });
        }
        events.forEach(function(e) { el.addEventListener(e, onInteraction, { once: true }); });
      } else if (config.priority === 'lazy') {
        if ('requestIdleCallback' in window) {
          requestIdleCallback(function() { hydrate(el); }, { timeout: config.timeout || 5000 });
        } else {
          setTimeout(function() { hydrate(el); }, 2000);
        }
      }

      // Fallback timeout
      if (config.fallback === 'immediate' && config.timeout) {
        setTimeout(function() { hydrate(el); }, parseInt(config.timeout));
      }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', processElements);
  } else {
    processElements();
  }
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_priority_ordering() {
        assert!(HydrationPriority::Immediate < HydrationPriority::OnIdle);
        assert!(HydrationPriority::OnIdle < HydrationPriority::OnVisible);
        assert!(HydrationPriority::OnVisible < HydrationPriority::OnInteraction);
        assert!(HydrationPriority::OnInteraction < HydrationPriority::Lazy);
    }

    #[test]
    fn test_hydration_priority_str() {
        assert_eq!(HydrationPriority::Immediate.as_str(), "immediate");
        assert_eq!(HydrationPriority::OnVisible.as_str(), "visible");
        assert_eq!(HydrationPriority::Lazy.as_str(), "lazy");
    }

    #[test]
    fn test_progressive_config_immediate() {
        let config = ProgressiveHydrationConfig::immediate();
        assert_eq!(config.priority, HydrationPriority::Immediate);
    }

    #[test]
    fn test_progressive_config_on_visible() {
        let config = ProgressiveHydrationConfig::on_visible("300px");
        assert_eq!(config.priority, HydrationPriority::OnVisible);
        assert_eq!(config.root_margin, "300px");
    }

    #[test]
    fn test_progressive_config_on_interaction() {
        let config = ProgressiveHydrationConfig::on_interaction(vec!["click".into(), "mouseover".into()]);
        assert_eq!(config.priority, HydrationPriority::OnInteraction);
        assert_eq!(config.interaction_events, vec!["click", "mouseover"]);
    }

    #[test]
    fn test_hydration_attr() {
        let config = ProgressiveHydrationConfig::on_visible("200px");
        let attr = hydration_attr(&config);
        assert!(attr.contains("visible"));
        assert!(attr.contains("rootMargin=200px"));
    }

    #[test]
    fn test_hydration_attr_immediate() {
        let config = ProgressiveHydrationConfig::immediate();
        let attr = hydration_attr(&config);
        assert!(attr.starts_with("immediate"));
    }

    #[test]
    fn test_progressive_hydration_script() {
        let script = progressive_hydration_script();
        assert!(script.contains("IntersectionObserver"));
        assert!(script.contains("requestIdleCallback"));
        assert!(script.contains("data-hydrate"));
    }
}
