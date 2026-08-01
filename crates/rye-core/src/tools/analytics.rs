//! Goal 139: Analytics events.
//! Goal 140: Web Vitals tracking.
//!
//! Privacy-first analytics: page views, custom events, and Core Web Vitals
//! (LCP, FID, CLS, INP, TTFB). No third-party scripts required.

use std::collections::HashMap;

/// An analytics event.
#[derive(Debug, Clone)]
pub struct AnalyticsEvent {
    /// Event name.
    pub name: String,
    /// Event category.
    pub category: String,
    /// Event properties.
    pub properties: HashMap<String, String>,
    /// Timestamp.
    pub timestamp: u64,
    /// User ID (if known).
    pub user_id: Option<String>,
    /// Session ID.
    pub session_id: String,
}

impl AnalyticsEvent {
    /// Create a new analytics event.
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            properties: HashMap::new(),
            timestamp: current_timestamp(),
            user_id: None,
            session_id: generate_session_id(),
        }
    }

    /// Add a property.
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Set user ID.
    pub fn user(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Convert to JSON.
    pub fn to_json(&self) -> String {
        let mut json = format!(
            r#"{{"name":"{}","category":"{}","timestamp":{},"session_id":"{}""#,
            escape(&self.name), escape(&self.category), self.timestamp, escape(&self.session_id)
        );

        if let Some(uid) = &self.user_id {
            json.push_str(&format!(r#","user_id":"{}""#, escape(uid)));
        }

        if !self.properties.is_empty() {
            let props: Vec<String> = self.properties.iter()
                .map(|(k, v)| format!(r#""{}":"{}""#, escape(k), escape(v)))
                .collect();
            json.push_str(&format!(r#","properties":{{{}}}"#, props.join(",")));
        }

        json.push('}');
        json
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn generate_session_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("session-{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Analytics tracker.
pub struct Analytics {
    /// Buffered events.
    events: Vec<AnalyticsEvent>,
    /// Whether to send events to a remote endpoint.
    pub endpoint: Option<String>,
    /// Batch size for remote sending.
    pub batch_size: usize,
}

impl Analytics {
    /// Create a new analytics tracker.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            endpoint: None,
            batch_size: 20,
        }
    }

    /// Set remote endpoint.
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Track a page view.
    pub fn page_view(&mut self, path: &str) {
        self.track(AnalyticsEvent::new("page_view", "navigation").prop("path", path));
    }

    /// Track a custom event.
    pub fn track(&mut self, event: AnalyticsEvent) {
        self.events.push(event);
    }

    /// Drain buffered events.
    pub fn drain(&mut self) -> Vec<AnalyticsEvent> {
        std::mem::take(&mut self.events)
    }

    /// Number of buffered events.
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for Analytics {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Web Vitals (Goal 140) =====

/// Core Web Vitals metrics.
#[derive(Debug, Clone, Default)]
pub struct WebVitals {
    /// Largest Contentful Paint (ms).
    pub lcp: Option<f64>,
    /// First Input Delay (ms).
    pub fid: Option<f64>,
    /// Cumulative Layout Shift (unitless).
    pub cls: Option<f64>,
    /// Interaction to Next Paint (ms).
    pub inp: Option<f64>,
    /// Time to First Byte (ms).
    pub ttfb: Option<f64>,
    /// First Contentful Paint (ms).
    pub fcp: Option<f64>,
    /// Time to Interactive (ms).
    pub tti: Option<f64>,
}

impl WebVitals {
    /// Get a rating for LCP.
    pub fn lcp_rating(&self) -> VitalRating {
        match self.lcp {
            Some(v) if v <= 2500.0 => VitalRating::Good,
            Some(v) if v <= 4000.0 => VitalRating::NeedsImprovement,
            Some(_) => VitalRating::Poor,
            None => VitalRating::Unknown,
        }
    }

    /// Get a rating for FID.
    pub fn fid_rating(&self) -> VitalRating {
        match self.fid {
            Some(v) if v <= 100.0 => VitalRating::Good,
            Some(v) if v <= 300.0 => VitalRating::NeedsImprovement,
            Some(_) => VitalRating::Poor,
            None => VitalRating::Unknown,
        }
    }

    /// Get a rating for CLS.
    pub fn cls_rating(&self) -> VitalRating {
        match self.cls {
            Some(v) if v <= 0.1 => VitalRating::Good,
            Some(v) if v <= 0.25 => VitalRating::NeedsImprovement,
            Some(_) => VitalRating::Poor,
            None => VitalRating::Unknown,
        }
    }

    /// Get a rating for INP.
    pub fn inp_rating(&self) -> VitalRating {
        match self.inp {
            Some(v) if v <= 200.0 => VitalRating::Good,
            Some(v) if v <= 500.0 => VitalRating::NeedsImprovement,
            Some(_) => VitalRating::Poor,
            None => VitalRating::Unknown,
        }
    }

    /// Get a rating for TTFB.
    pub fn ttfb_rating(&self) -> VitalRating {
        match self.ttfb {
            Some(v) if v <= 800.0 => VitalRating::Good,
            Some(v) if v <= 1800.0 => VitalRating::NeedsImprovement,
            Some(_) => VitalRating::Poor,
            None => VitalRating::Unknown,
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        let mut parts = Vec::new();
        if let Some(v) = self.lcp { parts.push(format!(r#""lcp":{}"#, v)); }
        if let Some(v) = self.fid { parts.push(format!(r#""fid":{}"#, v)); }
        if let Some(v) = self.cls { parts.push(format!(r#""cls":{}"#, v)); }
        if let Some(v) = self.inp { parts.push(format!(r#""inp":{}"#, v)); }
        if let Some(v) = self.ttfb { parts.push(format!(r#""ttfb":{}"#, v)); }
        if let Some(v) = self.fcp { parts.push(format!(r#""fcp":{}"#, v)); }
        if let Some(v) = self.tti { parts.push(format!(r#""tti":{}"#, v)); }
        format!("{{{}}}", parts.join(","))
    }
}

/// Web Vital rating.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VitalRating {
    /// Good (green).
    Good,
    /// Needs improvement (orange).
    NeedsImprovement,
    /// Poor (red).
    Poor,
    /// Not measured.
    Unknown,
}

impl VitalRating {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            VitalRating::Good => "good",
            VitalRating::NeedsImprovement => "needs-improvement",
            VitalRating::Poor => "poor",
            VitalRating::Unknown => "unknown",
        }
    }
}

/// Generate the JS for Web Vitals tracking.
pub fn web_vitals_script() -> &'static str {
    r#"<script>
(function() {
  var vitals = {};

  // LCP (Largest Contentful Paint)
  new PerformanceObserver(function(list) {
    var entries = list.getEntries();
    if (entries.length > 0) {
      vitals.lcp = entries[entries.length - 1].startTime;
    }
  }).observe({ type: 'largest-contentful-paint', buffered: true });

  // FID (First Input Delay)
  new PerformanceObserver(function(list) {
    var entries = list.getEntries();
    if (entries.length > 0) {
      vitals.fid = entries[0].processingStart - entries[0].startTime;
    }
  }).observe({ type: 'first-input', buffered: true });

  // CLS (Cumulative Layout Shift)
  var clsValue = 0;
  new PerformanceObserver(function(list) {
    list.getEntries().forEach(function(entry) {
      if (!entry.hadRecentInput) {
        clsValue += entry.value;
      }
    });
    vitals.cls = clsValue;
  }).observe({ type: 'layout-shift', buffered: true });

  // INP (Interaction to Next Paint)
  var maxInp = 0;
  new PerformanceObserver(function(list) {
    list.getEntries().forEach(function(entry) {
      var duration = entry.duration;
      if (duration > maxInp) maxInp = duration;
    });
    vitals.inp = maxInp;
  }).observe({ type: 'event', buffered: true });

  // TTFB (Time to First Byte)
  var navEntry = performance.getEntriesByType('navigation')[0];
  if (navEntry) {
    vitals.ttfb = navEntry.responseStart - navEntry.requestStart;
  }

  // FCP (First Contentful Paint)
  new PerformanceObserver(function(list) {
    var entries = list.getEntries();
    if (entries.length > 0) {
      vitals.fcp = entries[0].startTime;
    }
  }).observe({ type: 'paint', buffered: true });

  window.__rye_get_vitals = function() { return vitals; };

  // Send vitals on page unload
  window.addEventListener('pagehide', function() {
    if (navigator.sendBeacon && window.__rye_vitals_endpoint) {
      navigator.sendBeacon(window.__rye_vitals_endpoint, JSON.stringify(vitals));
    }
  });
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_event() {
        let event = AnalyticsEvent::new("click", "user_interaction")
            .prop("button", "submit")
            .prop("page", "/checkout")
            .user("user123");
        let json = event.to_json();
        assert!(json.contains("click"));
        assert!(json.contains("user_interaction"));
        assert!(json.contains("user123"));
        assert!(json.contains("button"));
    }

    #[test]
    fn test_analytics_page_view() {
        let mut analytics = Analytics::new();
        analytics.page_view("/home");
        analytics.page_view("/about");
        assert_eq!(analytics.len(), 2);
    }

    #[test]
    fn test_analytics_drain() {
        let mut analytics = Analytics::new();
        analytics.track(AnalyticsEvent::new("test", "test"));
        let drained = analytics.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(analytics.len(), 0);
    }

    #[test]
    fn test_web_vitals_lcp_rating() {
        let mut vitals = WebVitals::default();
        vitals.lcp = Some(2000.0);
        assert_eq!(vitals.lcp_rating(), VitalRating::Good);

        vitals.lcp = Some(3000.0);
        assert_eq!(vitals.lcp_rating(), VitalRating::NeedsImprovement);

        vitals.lcp = Some(5000.0);
        assert_eq!(vitals.lcp_rating(), VitalRating::Poor);
    }

    #[test]
    fn test_web_vitals_cls_rating() {
        let mut vitals = WebVitals::default();
        vitals.cls = Some(0.05);
        assert_eq!(vitals.cls_rating(), VitalRating::Good);

        vitals.cls = Some(0.15);
        assert_eq!(vitals.cls_rating(), VitalRating::NeedsImprovement);

        vitals.cls = Some(0.30);
        assert_eq!(vitals.cls_rating(), VitalRating::Poor);
    }

    #[test]
    fn test_web_vitals_inp_rating() {
        let mut vitals = WebVitals::default();
        vitals.inp = Some(150.0);
        assert_eq!(vitals.inp_rating(), VitalRating::Good);

        vitals.inp = Some(300.0);
        assert_eq!(vitals.inp_rating(), VitalRating::NeedsImprovement);

        vitals.inp = Some(600.0);
        assert_eq!(vitals.inp_rating(), VitalRating::Poor);
    }

    #[test]
    fn test_web_vitals_ttfb_rating() {
        let mut vitals = WebVitals::default();
        vitals.ttfb = Some(500.0);
        assert_eq!(vitals.ttfb_rating(), VitalRating::Good);

        vitals.ttfb = Some(1200.0);
        assert_eq!(vitals.ttfb_rating(), VitalRating::NeedsImprovement);

        vitals.ttfb = Some(2000.0);
        assert_eq!(vitals.ttfb_rating(), VitalRating::Poor);
    }

    #[test]
    fn test_web_vitals_json() {
        let mut vitals = WebVitals::default();
        vitals.lcp = Some(2000.0);
        vitals.cls = Some(0.1);
        let json = vitals.to_json();
        assert!(json.contains("lcp"));
        assert!(json.contains("2000"));
        assert!(json.contains("cls"));
    }

    #[test]
    fn test_vital_rating_str() {
        assert_eq!(VitalRating::Good.as_str(), "good");
        assert_eq!(VitalRating::Poor.as_str(), "poor");
    }

    #[test]
    fn test_web_vitals_script() {
        let script = web_vitals_script();
        assert!(script.contains("PerformanceObserver"));
        assert!(script.contains("largest-contentful-paint"));
        assert!(script.contains("layout-shift"));
        assert!(script.contains("__rye_get_vitals"));
    }
}
