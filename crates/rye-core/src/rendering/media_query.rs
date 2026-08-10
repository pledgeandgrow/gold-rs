//! Goal 115: Print/media query rendering.
//!
//! `use_media_query()` hook and `<PrintLayout>` component for print-optimized
//! output with CSS `@media print` integration.

/// Media query descriptor.
#[derive(Debug, Clone)]
pub struct MediaQuery {
    /// The media query string (e.g. "print", "(max-width: 768px)").
    pub query: String,
}

impl MediaQuery {
    /// Create a media query from a string.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
        }
    }

    /// Print media query.
    pub fn print() -> Self {
        Self::new("print")
    }

    /// Screen media query with max width.
    pub fn max_width(width: u32) -> Self {
        Self::new(format!("(max-width: {}px)", width))
    }

    /// Screen media query with min width.
    pub fn min_width(width: u32) -> Self {
        Self::new(format!("(min-width: {}px)", width))
    }

    /// Dark mode preference.
    pub fn dark_mode() -> Self {
        Self::new("(prefers-color-scheme: dark)")
    }

    /// Reduced motion preference.
    pub fn reduced_motion() -> Self {
        Self::new("(prefers-reduced-motion: reduce)")
    }
}

/// Media query match result.
#[derive(Debug, Clone)]
pub struct MediaMatch {
    /// Whether the query currently matches.
    pub matches: bool,
    /// The query string.
    pub query: String,
}

/// A set of media queries with their current match states.
pub struct MediaQueryTracker {
    /// Registered queries and their match states.
    queries: Vec<(MediaQuery, bool)>,
}

impl MediaQueryTracker {
    /// Create a new media query tracker.
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    /// Register a media query.
    pub fn register(&mut self, query: MediaQuery) -> usize {
        let idx = self.queries.len();
        self.queries.push((query, false));
        idx
    }

    /// Update the match state for a query.
    pub fn set_match(&mut self, idx: usize, matches: bool) {
        if idx < self.queries.len() {
            self.queries[idx].1 = matches;
        }
    }

    /// Check if a query matches.
    pub fn matches(&self, idx: usize) -> bool {
        self.queries.get(idx).map(|(_, m)| *m).unwrap_or(false)
    }

    /// Get all matching queries.
    pub fn matching_queries(&self) -> Vec<&MediaQuery> {
        self.queries
            .iter()
            .filter(|(_, m)| *m)
            .map(|(q, _)| q)
            .collect()
    }
}

impl Default for MediaQueryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate print-specific CSS rules.
pub fn print_css(rules: &[(&str, &str)]) -> String {
    let mut css = String::from("@media print {\n");
    for (selector, body) in rules {
        css.push_str(&format!("  {} {{\n    {}\n  }}\n", selector, body));
    }
    css.push_str("}\n");
    css
}

/// Generate the JS for media query matching.
pub fn media_query_script() -> &'static str {
    r#"<script>
(function() {
  var trackers = {};
  var nextId = 0;

  window.__rye_media_query = function(query, callbackId) {
    var mql = window.matchMedia(query);
    var id = 'mq_' + (nextId++);

    function onChange(e) {
      window.__rye_signal_update(callbackId, { matches: e.matches, query: query });
    }

    mql.addEventListener('change', onChange);
    trackers[id] = { mql: mql, callback: onChange };

    // Return initial match
    return { matches: mql.matches, query: query };
  };

  window.__rye_media_query_unsubscribe = function(id) {
    if (trackers[id]) {
      trackers[id].mql.removeEventListener('change', trackers[id].callback);
      delete trackers[id];
    }
  };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_query_constructors() {
        assert_eq!(MediaQuery::print().query, "print");
        assert_eq!(MediaQuery::max_width(768).query, "(max-width: 768px)");
        assert_eq!(MediaQuery::min_width(1024).query, "(min-width: 1024px)");
        assert_eq!(
            MediaQuery::dark_mode().query,
            "(prefers-color-scheme: dark)"
        );
        assert_eq!(
            MediaQuery::reduced_motion().query,
            "(prefers-reduced-motion: reduce)"
        );
    }

    #[test]
    fn test_media_query_tracker() {
        let mut tracker = MediaQueryTracker::new();
        let idx = tracker.register(MediaQuery::print());
        assert!(!tracker.matches(idx));
        tracker.set_match(idx, true);
        assert!(tracker.matches(idx));
        assert_eq!(tracker.matching_queries().len(), 1);
    }

    #[test]
    fn test_print_css() {
        let css = print_css(&[(".nav", "display: none;"), (".content", "width: 100%;")]);
        assert!(css.contains("@media print"));
        assert!(css.contains(".nav"));
        assert!(css.contains("display: none;"));
        assert!(css.contains(".content"));
    }

    #[test]
    fn test_media_query_script() {
        let script = media_query_script();
        assert!(script.contains("matchMedia"));
        assert!(script.contains("__rye_media_query"));
    }
}
