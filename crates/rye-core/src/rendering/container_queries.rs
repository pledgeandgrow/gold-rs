//! Goal 117: Container queries.
//!
//! `use_container_query()` hook for responsive components based on parent
//! container size, not viewport. CSS `@container` integration.

/// Container query descriptor.
#[derive(Debug, Clone)]
pub struct ContainerQuery {
    /// The container name (optional, for named containers).
    pub name: Option<String>,
    /// The query condition (e.g. "(min-width: 400px)").
    pub condition: String,
}

impl ContainerQuery {
    /// Create a container query with a condition.
    pub fn new(condition: impl Into<String>) -> Self {
        Self {
            name: None,
            condition: condition.into(),
        }
    }

    /// Create a named container query.
    pub fn named(name: impl Into<String>, condition: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            condition: condition.into(),
        }
    }

    /// Min width container query.
    pub fn min_width(width: u32) -> Self {
        Self::new(format!("(min-width: {}px)", width))
    }

    /// Max width container query.
    pub fn max_width(width: u32) -> Self {
        Self::new(format!("(max-width: {}px)", width))
    }
}

/// Container query match result.
#[derive(Debug, Clone)]
pub struct ContainerMatch {
    /// Whether the query matches.
    pub matches: bool,
    /// The container query.
    pub query: ContainerQuery,
}

/// Container type for `container-type` CSS property.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerType {
    /// Size container (both inline and block).
    Size,
    /// Inline size container.
    InlineSize,
    /// Block size container.
    BlockSize,
}

impl ContainerType {
    /// Convert to CSS string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerType::Size => "size",
            ContainerType::InlineSize => "inline-size",
            ContainerType::BlockSize => "block-size",
        }
    }
}

/// Generate CSS for a container with the given type and optional name.
pub fn container_css(selector: &str, container_type: ContainerType, name: Option<&str>) -> String {
    let mut css = format!(
        "{} {{\n  container-type: {};\n",
        selector,
        container_type.as_str()
    );
    if let Some(name) = name {
        css.push_str(&format!("  container-name: {};\n", name));
    }
    css.push_str("}\n");
    css
}

/// Generate CSS for a `@container` rule.
pub fn container_query_css(query: &ContainerQuery, rules: &[(&str, &str)]) -> String {
    let prefix = match &query.name {
        Some(name) => format!("@container {} {}", name, query.condition),
        None => format!("@container {}", query.condition),
    };

    let mut css = format!("{} {{\n", prefix);
    for (selector, body) in rules {
        css.push_str(&format!("  {} {{\n    {}\n  }}\n", selector, body));
    }
    css.push_str("}\n");
    css
}

/// Generate the JS for container query matching (using ResizeObserver).
pub fn container_query_script() -> &'static str {
    r#"<script>
(function() {
  // Container queries are supported natively in modern browsers.
  // For older browsers, we polyfill using ResizeObserver.
  var supportsContainerQueries = CSS.supports('container-type: size');

  window.__rye_container_query_supported = function() {
    return supportsContainerQueries;
  };

  if (!supportsContainerQueries) {
    var watchers = {};
    var nextId = 0;

    window.__rye_container_query_watch = function(elementId, condition, callbackId) {
      var el = document.getElementById(elementId);
      if (!el) return;

      var id = 'cq_' + (nextId++);
      var observer = new ResizeObserver(function(entries) {
        var entry = entries[0];
        var width = entry.contentRect.width;
        // Parse simple (min-width: Npx) / (max-width: Npx) conditions
        var minMatch = condition.match(/min-width:\s*(\d+)px/);
        var maxMatch = condition.match(/max-width:\s*(\d+)px/);
        var matches = true;
        if (minMatch) matches = matches && width >= parseInt(minMatch[1]);
        if (maxMatch) matches = matches && width <= parseInt(maxMatch[1]);
        window.__rye_signal_update(callbackId, { matches: matches });
      });
      observer.observe(el);
      watchers[id] = observer;
      return id;
    };

    window.__rye_container_query_unwatch = function(id) {
      if (watchers[id]) {
        watchers[id].disconnect();
        delete watchers[id];
      }
    };
  }
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_query_constructors() {
        let q = ContainerQuery::min_width(400);
        assert_eq!(q.condition, "(min-width: 400px)");
        assert!(q.name.is_none());

        let q = ContainerQuery::max_width(800);
        assert_eq!(q.condition, "(max-width: 800px)");

        let q = ContainerQuery::named("sidebar", "(min-width: 200px)");
        assert_eq!(q.name, Some("sidebar".to_string()));
    }

    #[test]
    fn test_container_type() {
        assert_eq!(ContainerType::Size.as_str(), "size");
        assert_eq!(ContainerType::InlineSize.as_str(), "inline-size");
        assert_eq!(ContainerType::BlockSize.as_str(), "block-size");
    }

    #[test]
    fn test_container_css() {
        let css = container_css(".card", ContainerType::InlineSize, Some("card"));
        assert!(css.contains("container-type: inline-size"));
        assert!(css.contains("container-name: card"));
    }

    #[test]
    fn test_container_css_no_name() {
        let css = container_css(".widget", ContainerType::Size, None);
        assert!(css.contains("container-type: size"));
        assert!(!css.contains("container-name"));
    }

    #[test]
    fn test_container_query_css() {
        let q = ContainerQuery::named("sidebar", "(min-width: 200px)");
        let css = container_query_css(&q, &[(".content", "flex-direction: row;")]);
        assert!(css.contains("@container sidebar (min-width: 200px)"));
        assert!(css.contains(".content"));
        assert!(css.contains("flex-direction: row;"));
    }

    #[test]
    fn test_container_query_script() {
        let script = container_query_script();
        assert!(script.contains("container-type"));
        assert!(script.contains("__rye_container_query"));
    }
}
