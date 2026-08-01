//! Goal 210: Native widget / live activity support.
//!
//! iOS Live Activities and Android App Widgets. Define widget UI with rye components,
//! render to platform widget format. Data binding via signals.

use std::collections::HashMap;
use std::sync::Mutex;

/// The widget platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetPlatform {
    /// iOS Live Activity / WidgetKit.
    Ios,
    /// Android App Widget (Glance).
    Android,
}

impl WidgetPlatform {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            WidgetPlatform::Ios => "iOS WidgetKit",
            WidgetPlatform::Android => "Android App Widget",
        }
    }
}

/// The widget size family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetSize {
    /// Small widget.
    Small,
    /// Medium widget.
    Medium,
    /// Large widget.
    Large,
    /// Extra large widget (iOS only).
    ExtraLarge,
    /// Lock screen widget (iOS).
    LockScreen,
}

impl WidgetSize {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            WidgetSize::Small => "Small",
            WidgetSize::Medium => "Medium",
            WidgetSize::Large => "Large",
            WidgetSize::ExtraLarge => "Extra Large",
            WidgetSize::LockScreen => "Lock Screen",
        }
    }

    /// Get the supported platforms for this size.
    pub fn supported_platforms(&self) -> &'static [WidgetPlatform] {
        match self {
            WidgetSize::ExtraLarge | WidgetSize::LockScreen => &[WidgetPlatform::Ios],
            _ => &[WidgetPlatform::Ios, WidgetPlatform::Android],
        }
    }
}

/// A widget data binding — maps a signal to a widget UI element.
#[derive(Debug, Clone)]
pub struct WidgetBinding {
    /// The binding key (matches a signal ID).
    pub key: String,
    /// The display label.
    pub label: String,
    /// The default value.
    pub default_value: String,
    /// Whether the value can be updated while the widget is displayed.
    pub updatable: bool,
}

impl WidgetBinding {
    /// Create a new widget binding.
    pub fn new(key: &str, default_value: &str) -> Self {
        Self {
            key: key.to_string(),
            label: key.to_string(),
            default_value: default_value.to_string(),
            updatable: true,
        }
    }

    /// Set the label.
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    /// Make non-updatable (static value).
    pub fn static_value(mut self) -> Self {
        self.updatable = false;
        self
    }
}

/// A widget definition — describes a native widget.
#[derive(Debug, Clone)]
pub struct WidgetDefinition {
    /// The widget identifier.
    pub id: String,
    /// The widget display name.
    pub name: String,
    /// The widget description.
    pub description: String,
    /// The target platform.
    pub platform: WidgetPlatform,
    /// The supported sizes.
    pub sizes: Vec<WidgetSize>,
    /// The data bindings.
    pub bindings: Vec<WidgetBinding>,
    /// The update interval in seconds (minimum).
    pub min_update_interval: u64,
}

impl WidgetDefinition {
    /// Create a new widget definition.
    pub fn new(id: &str, name: &str, platform: WidgetPlatform) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            platform,
            sizes: vec![WidgetSize::Small, WidgetSize::Medium],
            bindings: Vec::new(),
            min_update_interval: 1800, // 30 minutes minimum
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the supported sizes.
    pub fn with_sizes(mut self, sizes: &[WidgetSize]) -> Self {
        self.sizes = sizes.to_vec();
        self
    }

    /// Add a data binding.
    pub fn add_binding(mut self, binding: WidgetBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    /// Set the minimum update interval.
    pub fn with_update_interval(mut self, seconds: u64) -> Self {
        self.min_update_interval = seconds;
        self
    }

    /// Get a binding by key.
    pub fn get_binding(&self, key: &str) -> Option<&WidgetBinding> {
        self.bindings.iter().find(|b| b.key == key)
    }

    /// Check if a size is supported.
    pub fn supports_size(&self, size: WidgetSize) -> bool {
        self.sizes.contains(&size)
    }

    /// Get the number of bindings.
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }
}

/// The current state of a widget instance.
#[derive(Debug, Clone)]
pub struct WidgetState {
    /// The widget definition ID.
    pub widget_id: String,
    /// The current size.
    pub size: WidgetSize,
    /// The current binding values.
    pub values: HashMap<String, String>,
    /// Whether the widget is currently visible.
    pub visible: bool,
}

impl WidgetState {
    /// Create a new widget state.
    pub fn new(widget_id: &str, size: WidgetSize) -> Self {
        Self {
            widget_id: widget_id.to_string(),
            size,
            values: HashMap::new(),
            visible: true,
        }
    }

    /// Set a binding value.
    pub fn set_value(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Get a binding value.
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// The widget manager — registers widgets and manages their state.
pub struct WidgetManager {
    widgets: Mutex<HashMap<String, WidgetDefinition>>,
    states: Mutex<HashMap<String, WidgetState>>,
    update_count: Mutex<u32>,
}

impl WidgetManager {
    /// Create a new widget manager.
    pub fn new() -> Self {
        Self {
            widgets: Mutex::new(HashMap::new()),
            states: Mutex::new(HashMap::new()),
            update_count: Mutex::new(0),
        }
    }

    /// Register a widget definition.
    pub fn register(&self, widget: WidgetDefinition) {
        self.widgets.lock().unwrap().insert(widget.id.clone(), widget);
    }

    /// Get a widget definition by ID.
    pub fn get_widget(&self, id: &str) -> Option<WidgetDefinition> {
        self.widgets.lock().unwrap().get(id).cloned()
    }

    /// Get all registered widget IDs.
    pub fn widget_ids(&self) -> Vec<String> {
        self.widgets.lock().unwrap().keys().cloned().collect()
    }

    /// Get the number of registered widgets.
    pub fn widget_count(&self) -> usize {
        self.widgets.lock().unwrap().len()
    }

    /// Create a widget instance with a state.
    pub fn create_instance(&self, widget_id: &str, size: WidgetSize) -> Option<String> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)?;
        if !widget.supports_size(size) {
            return None;
        }
        drop(widgets);

        let instance_id = format!("{}_{}_{}", widget_id, format!("{:?}", size), self.states.lock().unwrap().len());
        let mut state = WidgetState::new(widget_id, size);

        // Initialize with default values from bindings
        let widgets = self.widgets.lock().unwrap();
        if let Some(widget) = widgets.get(widget_id) {
            for binding in &widget.bindings {
                state.set_value(&binding.key, &binding.default_value);
            }
        }
        drop(widgets);

        self.states.lock().unwrap().insert(instance_id.clone(), state);
        Some(instance_id)
    }

    /// Update a widget instance's binding value.
    pub fn update_value(&self, instance_id: &str, key: &str, value: &str) -> bool {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.get_mut(instance_id) {
            state.set_value(key, value);
            *self.update_count.lock().unwrap() += 1;
            return true;
        }
        false
    }

    /// Get a widget instance state.
    pub fn get_state(&self, instance_id: &str) -> Option<WidgetState> {
        self.states.lock().unwrap().get(instance_id).cloned()
    }

    /// Remove a widget instance.
    pub fn remove_instance(&self, instance_id: &str) -> bool {
        self.states.lock().unwrap().remove(instance_id).is_some()
    }

    /// Get the number of active instances.
    pub fn instance_count(&self) -> usize {
        self.states.lock().unwrap().len()
    }

    /// Get the update count.
    pub fn update_count(&self) -> u32 {
        *self.update_count.lock().unwrap()
    }

    /// Unregister a widget definition.
    pub fn unregister(&self, widget_id: &str) -> bool {
        self.widgets.lock().unwrap().remove(widget_id).is_some()
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_platform_display_name() {
        assert_eq!(WidgetPlatform::Ios.display_name(), "iOS WidgetKit");
        assert_eq!(WidgetPlatform::Android.display_name(), "Android App Widget");
    }

    #[test]
    fn test_widget_size_display_name() {
        assert_eq!(WidgetSize::Small.display_name(), "Small");
        assert_eq!(WidgetSize::Large.display_name(), "Large");
        assert_eq!(WidgetSize::LockScreen.display_name(), "Lock Screen");
    }

    #[test]
    fn test_widget_size_supported_platforms() {
        assert_eq!(WidgetSize::Small.supported_platforms().len(), 2);
        assert_eq!(WidgetSize::ExtraLarge.supported_platforms().len(), 1);
        assert_eq!(WidgetSize::LockScreen.supported_platforms().len(), 1);
    }

    #[test]
    fn test_widget_binding_new() {
        let b = WidgetBinding::new("temp", "72");
        assert_eq!(b.key, "temp");
        assert_eq!(b.default_value, "72");
        assert!(b.updatable);
    }

    #[test]
    fn test_widget_binding_builder() {
        let b = WidgetBinding::new("temp", "72")
            .with_label("Temperature")
            .static_value();
        assert_eq!(b.label, "Temperature");
        assert!(!b.updatable);
    }

    #[test]
    fn test_widget_definition_new() {
        let w = WidgetDefinition::new("weather", "Weather", WidgetPlatform::Ios);
        assert_eq!(w.id, "weather");
        assert_eq!(w.name, "Weather");
        assert_eq!(w.platform, WidgetPlatform::Ios);
        assert!(w.supports_size(WidgetSize::Small));
        assert!(!w.supports_size(WidgetSize::ExtraLarge));
    }

    #[test]
    fn test_widget_definition_builder() {
        let w = WidgetDefinition::new("weather", "Weather", WidgetPlatform::Ios)
            .with_description("Show weather")
            .with_sizes(&[WidgetSize::Medium, WidgetSize::Large])
            .add_binding(WidgetBinding::new("temp", "72"))
            .add_binding(WidgetBinding::new("city", "SF"))
            .with_update_interval(900);

        assert_eq!(w.description, "Show weather");
        assert!(!w.supports_size(WidgetSize::Small));
        assert!(w.supports_size(WidgetSize::Medium));
        assert_eq!(w.binding_count(), 2);
        assert_eq!(w.min_update_interval, 900);
    }

    #[test]
    fn test_widget_definition_get_binding() {
        let w = WidgetDefinition::new("w", "W", WidgetPlatform::Android)
            .add_binding(WidgetBinding::new("key1", "val1"));
        assert!(w.get_binding("key1").is_some());
        assert!(w.get_binding("nonexistent").is_none());
    }

    #[test]
    fn test_widget_state_new() {
        let s = WidgetState::new("widget1", WidgetSize::Small);
        assert_eq!(s.widget_id, "widget1");
        assert_eq!(s.size, WidgetSize::Small);
        assert!(s.visible);
    }

    #[test]
    fn test_widget_state_values() {
        let mut s = WidgetState::new("w", WidgetSize::Medium);
        s.set_value("temp", "72");
        s.set_value("city", "SF");
        assert_eq!(s.get_value("temp"), Some("72"));
        assert_eq!(s.get_value("city"), Some("SF"));
        assert_eq!(s.get_value("nonexistent"), None);
    }

    #[test]
    fn test_widget_state_visibility() {
        let mut s = WidgetState::new("w", WidgetSize::Small);
        s.set_visible(false);
        assert!(!s.visible);
    }

    #[test]
    fn test_manager_register_get() {
        let mgr = WidgetManager::new();
        mgr.register(WidgetDefinition::new("weather", "Weather", WidgetPlatform::Ios));
        assert!(mgr.get_widget("weather").is_some());
        assert!(mgr.get_widget("nonexistent").is_none());
        assert_eq!(mgr.widget_count(), 1);
    }

    #[test]
    fn test_manager_widget_ids() {
        let mgr = WidgetManager::new();
        mgr.register(WidgetDefinition::new("a", "A", WidgetPlatform::Ios));
        mgr.register(WidgetDefinition::new("b", "B", WidgetPlatform::Android));
        assert_eq!(mgr.widget_ids().len(), 2);
    }

    #[test]
    fn test_manager_create_instance() {
        let mgr = WidgetManager::new();
        mgr.register(
            WidgetDefinition::new("weather", "Weather", WidgetPlatform::Ios)
                .add_binding(WidgetBinding::new("temp", "72")),
        );
        let instance_id = mgr.create_instance("weather", WidgetSize::Small);
        assert!(instance_id.is_some());
        assert_eq!(mgr.instance_count(), 1);
    }

    #[test]
    fn test_manager_create_instance_unsupported_size() {
        let mgr = WidgetManager::new();
        mgr.register(
            WidgetDefinition::new("w", "W", WidgetPlatform::Android)
                .with_sizes(&[WidgetSize::Small]),
        );
        let instance_id = mgr.create_instance("w", WidgetSize::Large);
        assert!(instance_id.is_none());
    }

    #[test]
    fn test_manager_create_instance_defaults() {
        let mgr = WidgetManager::new();
        mgr.register(
            WidgetDefinition::new("w", "W", WidgetPlatform::Ios)
                .add_binding(WidgetBinding::new("temp", "72")),
        );
        let instance_id = mgr.create_instance("w", WidgetSize::Small).unwrap();
        let state = mgr.get_state(&instance_id).unwrap();
        assert_eq!(state.get_value("temp"), Some("72"));
    }

    #[test]
    fn test_manager_update_value() {
        let mgr = WidgetManager::new();
        mgr.register(WidgetDefinition::new("w", "W", WidgetPlatform::Ios));
        let instance_id = mgr.create_instance("w", WidgetSize::Small).unwrap();
        assert!(mgr.update_value(&instance_id, "temp", "80"));
        assert_eq!(mgr.get_state(&instance_id).unwrap().get_value("temp"), Some("80"));
        assert_eq!(mgr.update_count(), 1);
    }

    #[test]
    fn test_manager_update_value_nonexistent() {
        let mgr = WidgetManager::new();
        assert!(!mgr.update_value("nonexistent", "key", "val"));
    }

    #[test]
    fn test_manager_remove_instance() {
        let mgr = WidgetManager::new();
        mgr.register(WidgetDefinition::new("w", "W", WidgetPlatform::Ios));
        let id = mgr.create_instance("w", WidgetSize::Small).unwrap();
        assert!(mgr.remove_instance(&id));
        assert_eq!(mgr.instance_count(), 0);
    }

    #[test]
    fn test_manager_unregister() {
        let mgr = WidgetManager::new();
        mgr.register(WidgetDefinition::new("w", "W", WidgetPlatform::Ios));
        assert!(mgr.unregister("w"));
        assert_eq!(mgr.widget_count(), 0);
    }
}
