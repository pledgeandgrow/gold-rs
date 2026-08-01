//! Goal 134: Storybook integration.
//!
//! `rye storybook` command that generates a Storybook-style component
//! playground. Stories are defined with a simple macro and rendered in
//! isolation with hot reload.

use std::collections::HashMap;

/// A story definition.
#[derive(Debug, Clone)]
pub struct Story {
    /// Story name.
    pub name: String,
    /// Component name.
    pub component: String,
    /// Props as key-value pairs.
    pub props: HashMap<String, String>,
    /// Display name for the story.
    pub display_name: String,
}

impl Story {
    /// Create a new story.
    pub fn new(component: impl Into<String>, name: impl Into<String>) -> Self {
        let component = component.into();
        let name = name.into();
        Self {
            display_name: format!("{}: {}", component, name),
            name,
            component,
            props: HashMap::new(),
        }
    }

    /// Add a prop.
    pub fn prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }
}

/// A component's stories collection.
#[derive(Debug, Clone)]
pub struct ComponentStories {
    /// Component name.
    pub component: String,
    /// Stories for this component.
    pub stories: Vec<Story>,
    /// Default props.
    pub default_props: HashMap<String, String>,
}

/// Storybook configuration.
#[derive(Debug, Clone)]
pub struct StorybookConfig {
    /// Title for the storybook.
    pub title: String,
    /// Component stories.
    pub components: Vec<ComponentStories>,
    /// Whether to enable hot reload.
    pub hot_reload: bool,
    /// Port for the storybook server.
    pub port: u16,
}

impl Default for StorybookConfig {
    fn default() -> Self {
        Self {
            title: "Rye Storybook".to_string(),
            components: Vec::new(),
            hot_reload: true,
            port: 6006,
        }
    }
}

impl StorybookConfig {
    /// Create a new storybook config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Add component stories.
    pub fn component(mut self, stories: ComponentStories) -> Self {
        self.components.push(stories);
        self
    }
}

/// Generate the storybook HTML page.
pub fn storybook_html(config: &StorybookConfig) -> String {
    let mut stories_json = Vec::new();
    for comp in &config.components {
        let stories: Vec<String> = comp.stories.iter().map(|s| {
            let props: Vec<String> = s.props.iter()
                .map(|(k, v)| format!(r#""{}":"{}""#, k, v))
                .collect();
            format!(
                r#"{{"name":"{}","props":{{{}}}}}"#,
                s.name,
                props.join(",")
            )
        }).collect();

        stories_json.push(format!(
            r#"{{"component":"{}","stories":[{}]}}"#,
            comp.component,
            stories.join(",")
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <title>{title}</title>
  <style>
    body {{ margin: 0; font-family: system-ui; }}
    .sidebar {{ width: 250px; height: 100vh; background: #f5f5f5; padding: 20px; overflow-y: auto; float: left; }}
    .main {{ margin-left: 270px; padding: 20px; }}
    .story-link {{ display: block; padding: 8px; cursor: pointer; border-radius: 4px; }}
    .story-link:hover {{ background: #e0e0e0; }}
    .story-link.active {{ background: #ddd; }}
    #story-container {{ border: 1px solid #ddd; min-height: 400px; padding: 20px; }}
  </style>
</head>
<body>
  <div class="sidebar">
    <h2>{title}</h2>
    <div id="story-list"></div>
  </div>
  <div class="main">
    <div id="story-container"></div>
  </div>
  <script>
    var stories = [{stories}];
    var currentStory = null;

    function renderStoryList() {{
      var list = document.getElementById('story-list');
      stories.forEach(function(comp) {{
        var h3 = document.createElement('h3');
        h3.textContent = comp.component;
        list.appendChild(h3);
        comp.stories.forEach(function(story) {{
          var link = document.createElement('a');
          link.className = 'story-link';
          link.textContent = story.name;
          link.onclick = function() {{
            currentStory = {{ component: comp.component, story: story }};
            renderStory();
          }};
          list.appendChild(link);
        }});
      }});
    }}

    function renderStory() {{
      var container = document.getElementById('story-container');
      container.innerHTML = '<p>Loading ' + currentStory.component + '...</p>';
      if (window.__rye_render_story) {{
        window.__rye_render_story(currentStory.component, currentStory.story.props, container);
      }}
    }}

    renderStoryList();
  </script>
</body>
</html>"#,
        title = config.title,
        stories = stories_json.join(",")
    )
}

/// Generate the storybook server script for hot reload.
pub fn storybook_hot_reload_script(port: u16) -> String {
    format!(
        r#"<script>
(function() {{
  var ws = new WebSocket('ws://localhost:{port}');
  ws.onmessage = function(event) {{
    if (event.data === 'reload') {{
      location.reload();
    }}
  }};
}})();
</script>"#,
        port = port
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_creation() {
        let story = Story::new("Button", "Primary")
            .prop("label", "Click me")
            .prop("variant", "primary");
        assert_eq!(story.component, "Button");
        assert_eq!(story.name, "Primary");
        assert_eq!(story.props.len(), 2);
    }

    #[test]
    fn test_storybook_config() {
        let config = StorybookConfig::new()
            .title("My Storybook")
            .port(8080);
        assert_eq!(config.title, "My Storybook");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_storybook_html() {
        let comp = ComponentStories {
            component: "Button".to_string(),
            stories: vec![
                Story::new("Button", "Primary").prop("label", "Click"),
                Story::new("Button", "Secondary"),
            ],
            default_props: HashMap::new(),
        };
        let config = StorybookConfig::new().component(comp);
        let html = storybook_html(&config);
        assert!(html.contains("My Storybook") || html.contains("Rye Storybook"));
        assert!(html.contains("Button"));
        assert!(html.contains("Primary"));
        assert!(html.contains("story-link"));
    }

    #[test]
    fn test_hot_reload_script() {
        let script = storybook_hot_reload_script(6006);
        assert!(script.contains("ws://localhost:6006"));
        assert!(script.contains("reload"));
    }
}
