//! Goal 113: Intersection observer abstraction.
//! Goal 114: Resize observer abstraction.
//!
//! `use_intersection()` and `use_resize()` hooks that wrap browser APIs
//! and return reactive signals.

/// Intersection observer configuration.
#[derive(Debug, Clone)]
pub struct IntersectionConfig {
    /// Root margin (CSS margin string).
    pub root_margin: String,
    /// Threshold(s) for triggering.
    pub thresholds: Vec<f32>,
}

impl Default for IntersectionConfig {
    fn default() -> Self {
        Self {
            root_margin: "0px".to_string(),
            thresholds: vec![0.0, 0.25, 0.5, 0.75, 1.0],
        }
    }
}

impl IntersectionConfig {
    /// Create a config that triggers when the element is fully visible.
    pub fn fully_visible() -> Self {
        Self {
            root_margin: "0px".to_string(),
            thresholds: vec![1.0],
        }
    }

    /// Create a config that triggers as soon as any part is visible.
    pub fn any_visible() -> Self {
        Self {
            root_margin: "0px".to_string(),
            thresholds: vec![0.0],
        }
    }

    /// Set a custom root margin.
    pub fn with_root_margin(mut self, margin: impl Into<String>) -> Self {
        self.root_margin = margin.into();
        self
    }
}

/// Intersection observer entry — visibility state for an element.
#[derive(Debug, Clone)]
pub struct IntersectionEntry {
    /// Whether the element is currently intersecting.
    pub is_intersecting: bool,
    /// Intersection ratio (0.0 to 1.0).
    pub intersection_ratio: f32,
    /// Bounding client rect of the element.
    pub bounding_rect: Rect,
}

/// A simple rectangle.
#[derive(Debug, Clone, Default)]
pub struct Rect {
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// Generate the JS for intersection observer.
pub fn intersection_observer_script() -> &'static str {
    r#"<script>
(function() {
  var observers = {};
  var nextId = 0;

  window.__rye_intersection_observe = function(elementId, callbackId, config) {
    var el = document.getElementById(elementId);
    if (!el) return;

    var obsId = 'iobs_' + (nextId++);
    var observer = new IntersectionObserver(function(entries) {
      var entry = entries[0];
      window.__rye_signal_update(callbackId, {
        isIntersecting: entry.isIntersecting,
        intersectionRatio: entry.intersectionRatio,
        boundingRect: {
          x: entry.boundingClientRect.x,
          y: entry.boundingClientRect.y,
          width: entry.boundingClientRect.width,
          height: entry.boundingClientRect.height
        }
      });
    }, {
      rootMargin: config.rootMargin || '0px',
      threshold: config.thresholds || [0]
    });

    observer.observe(el);
    observers[obsId] = observer;
    return obsId;
  };

  window.__rye_intersection_unobserve = function(obsId) {
    if (observers[obsId]) {
      observers[obsId].disconnect();
      delete observers[obsId];
    }
  };
})();
</script>"#
}

// ===== Resize Observer (Goal 114) =====

/// Resize observer entry.
#[derive(Debug, Clone)]
pub struct ResizeEntry {
    /// New content rect.
    pub content_rect: Rect,
    /// New border box size (width, height).
    pub border_box_size: (f64, f64),
}

/// Generate the JS for resize observer.
pub fn resize_observer_script() -> &'static str {
    r#"<script>
(function() {
  var observers = {};
  var nextId = 0;

  window.__rye_resize_observe = function(elementId, callbackId) {
    var el = document.getElementById(elementId);
    if (!el) return;

    var obsId = 'robs_' + (nextId++);
    var observer = new ResizeObserver(function(entries) {
      var entry = entries[0];
      window.__rye_signal_update(callbackId, {
        contentRect: {
          x: entry.contentRect.x,
          y: entry.contentRect.y,
          width: entry.contentRect.width,
          height: entry.contentRect.height
        },
        borderBoxSize: {
          width: entry.borderBoxSize[0].inlineSize,
          height: entry.borderBoxSize[0].blockSize
        }
      });
    });

    observer.observe(el);
    observers[obsId] = observer;
    return obsId;
  };

  window.__rye_resize_unobserve = function(obsId) {
    if (observers[obsId]) {
      observers[obsId].disconnect();
      delete observers[obsId];
    }
  };
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_config_default() {
        let config = IntersectionConfig::default();
        assert_eq!(config.root_margin, "0px");
        assert_eq!(config.thresholds.len(), 5);
    }

    #[test]
    fn test_intersection_config_fully_visible() {
        let config = IntersectionConfig::fully_visible();
        assert_eq!(config.thresholds, vec![1.0]);
    }

    #[test]
    fn test_intersection_config_any_visible() {
        let config = IntersectionConfig::any_visible();
        assert_eq!(config.thresholds, vec![0.0]);
    }

    #[test]
    fn test_intersection_config_root_margin() {
        let config = IntersectionConfig::default().with_root_margin("100px");
        assert_eq!(config.root_margin, "100px");
    }

    #[test]
    fn test_intersection_observer_script() {
        let script = intersection_observer_script();
        assert!(script.contains("IntersectionObserver"));
        assert!(script.contains("__rye_intersection_observe"));
        assert!(script.contains("__rye_intersection_unobserve"));
    }

    #[test]
    fn test_resize_observer_script() {
        let script = resize_observer_script();
        assert!(script.contains("ResizeObserver"));
        assert!(script.contains("__rye_resize_observe"));
        assert!(script.contains("__rye_resize_unobserve"));
    }

    #[test]
    fn test_rect_default() {
        let r = Rect::default();
        assert_eq!(r.x, 0.0);
        assert_eq!(r.width, 0.0);
    }
}
