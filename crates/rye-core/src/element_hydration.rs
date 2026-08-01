//! Element-level lazy hydration — hydrate individual DOM nodes as they become interactive.
//!
//! Extends progressive hydration to the element level — hydrate individual DOM nodes
//! as they become interactive, not just whole components. Finer granularity = faster TTI.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// The hydration strategy for an individual element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementHydrationStrategy {
    /// Hydrate immediately on page load.
    Immediate,
    /// Hydrate when the element is scrolled into view.
    OnVisible,
    /// Hydrate when the element receives focus or is interacted with.
    OnInteraction,
    /// Hydrate when the browser is idle.
    OnIdle,
    /// Hydrate after a delay (milliseconds).
    Delayed,
}

/// Configuration for element-level lazy hydration.
#[derive(Debug, Clone)]
pub struct ElementHydrationConfig {
    /// The strategy for this element.
    pub strategy: ElementHydrationStrategy,
    /// Delay in milliseconds (for Delayed strategy).
    pub delay_ms: u32,
    /// Root margin for OnVisible (CSS margin string).
    pub root_margin: String,
    /// Threshold for OnVisible (0.0 to 1.0).
    pub threshold: f64,
}

impl Default for ElementHydrationConfig {
    fn default() -> Self {
        Self {
            strategy: ElementHydrationStrategy::OnInteraction,
            delay_ms: 0,
            root_margin: "0px".to_string(),
            threshold: 0.0,
        }
    }
}

impl ElementHydrationConfig {
    /// Create an immediate hydration config.
    pub fn immediate() -> Self {
        Self {
            strategy: ElementHydrationStrategy::Immediate,
            ..Default::default()
        }
    }

    /// Create an on-visible hydration config.
    pub fn on_visible() -> Self {
        Self {
            strategy: ElementHydrationStrategy::OnVisible,
            threshold: 0.1,
            ..Default::default()
        }
    }

    /// Create an on-interaction hydration config.
    pub fn on_interaction() -> Self {
        Self {
            strategy: ElementHydrationStrategy::OnInteraction,
            ..Default::default()
        }
    }

    /// Create an on-idle hydration config.
    pub fn on_idle() -> Self {
        Self {
            strategy: ElementHydrationStrategy::OnIdle,
            ..Default::default()
        }
    }

    /// Create a delayed hydration config.
    pub fn delayed(delay_ms: u32) -> Self {
        Self {
            strategy: ElementHydrationStrategy::Delayed,
            delay_ms,
            ..Default::default()
        }
    }
}

/// The hydration state of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementHydrationState {
    /// Not yet hydrated.
    Pending,
    /// Currently hydrating.
    Hydrating,
    /// Fully hydrated and interactive.
    Hydrated,
}

/// An element registered for lazy hydration.
#[derive(Debug, Clone)]
pub struct LazyHydrationEntry {
    /// Unique element ID (matches hydration marker ID).
    pub element_id: usize,
    /// The hydration config for this element.
    pub config: ElementHydrationConfig,
    /// Current hydration state.
    pub state: ElementHydrationState,
    /// The component that owns this element.
    pub component_name: String,
}

/// The element-level hydration manager — tracks and hydrates individual elements.
pub struct ElementHydrationManager {
    entries: RefCell<HashMap<usize, LazyHydrationEntry>>,
    hydrated: RefCell<HashSet<usize>>,
    pending_callbacks: RefCell<HashMap<usize, Vec<Rc<dyn Fn()>>>>,
}

impl ElementHydrationManager {
    /// Create a new hydration manager.
    pub fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
            hydrated: RefCell::new(HashSet::new()),
            pending_callbacks: RefCell::new(HashMap::new()),
        }
    }

    /// Register an element for lazy hydration.
    pub fn register(&self, element_id: usize, config: ElementHydrationConfig, component_name: &str) {
        self.entries.borrow_mut().insert(
            element_id,
            LazyHydrationEntry {
                element_id,
                config,
                state: ElementHydrationState::Pending,
                component_name: component_name.to_string(),
            },
        );
    }

    /// Register a callback to run when an element is hydrated.
    pub fn on_hydrate<F: Fn() + 'static>(&self, element_id: usize, callback: F) {
        self.pending_callbacks
            .borrow_mut()
            .entry(element_id)
            .or_default()
            .push(Rc::new(callback));
    }

    /// Mark an element as visible (triggers OnVisible hydration).
    pub fn mark_visible(&self, element_id: usize) {
        let should_hydrate = {
            let entries = self.entries.borrow();
            entries.get(&element_id).map_or(false, |e| {
                e.config.strategy == ElementHydrationStrategy::OnVisible
                    && e.state == ElementHydrationState::Pending
            })
        };
        if should_hydrate {
            self.hydrate(element_id);
        }
    }

    /// Mark an element as interacted with (triggers OnInteraction hydration).
    pub fn mark_interaction(&self, element_id: usize) {
        let should_hydrate = {
            let entries = self.entries.borrow();
            entries.get(&element_id).map_or(false, |e| {
                e.config.strategy == ElementHydrationStrategy::OnInteraction
                    && e.state == ElementHydrationState::Pending
            })
        };
        if should_hydrate {
            self.hydrate(element_id);
        }
    }

    /// Mark the browser as idle (triggers OnIdle hydration).
    pub fn mark_idle(&self) {
        let to_hydrate: Vec<usize> = {
            let entries = self.entries.borrow();
            entries
                .iter()
                .filter(|(_, e)| {
                    e.config.strategy == ElementHydrationStrategy::OnIdle
                        && e.state == ElementHydrationState::Pending
                })
                .map(|(id, _)| *id)
                .collect()
        };
        for id in to_hydrate {
            self.hydrate(id);
        }
    }

    /// Hydrate all pending elements with Immediate strategy.
    pub fn hydrate_immediate(&self) {
        let to_hydrate: Vec<usize> = {
            let entries = self.entries.borrow();
            entries
                .iter()
                .filter(|(_, e)| {
                    e.config.strategy == ElementHydrationStrategy::Immediate
                        && e.state == ElementHydrationState::Pending
                })
                .map(|(id, _)| *id)
                .collect()
        };
        for id in to_hydrate {
            self.hydrate(id);
        }
    }

    /// Hydrate a specific element.
    pub fn hydrate(&self, element_id: usize) {
        // Check if already hydrated
        if self.hydrated.borrow().contains(&element_id) {
            return;
        }

        // Update state to Hydrating
        {
            let mut entries = self.entries.borrow_mut();
            if let Some(entry) = entries.get_mut(&element_id) {
                entry.state = ElementHydrationState::Hydrating;
            } else {
                return; // Not registered
            }
        }

        // Run callbacks
        let callbacks = self.pending_callbacks.borrow().get(&element_id).cloned();
        if let Some(callbacks) = callbacks {
            for cb in &callbacks {
                cb();
            }
        }

        // Update state to Hydrated
        {
            let mut entries = self.entries.borrow_mut();
            if let Some(entry) = entries.get_mut(&element_id) {
                entry.state = ElementHydrationState::Hydrated;
            }
        }
        self.hydrated.borrow_mut().insert(element_id);
        self.pending_callbacks.borrow_mut().remove(&element_id);
    }

    /// Hydrate all pending elements.
    pub fn hydrate_all(&self) {
        let ids: Vec<usize> = self.entries.borrow().keys().copied().collect();
        for id in ids {
            self.hydrate(id);
        }
    }

    /// Check if an element is hydrated.
    pub fn is_hydrated(&self, element_id: usize) -> bool {
        self.hydrated.borrow().contains(&element_id)
    }

    /// Get the hydration state of an element.
    pub fn state(&self, element_id: usize) -> Option<ElementHydrationState> {
        self.entries.borrow().get(&element_id).map(|e| e.state)
    }

    /// Get the number of registered elements.
    pub fn registered_count(&self) -> usize {
        self.entries.borrow().len()
    }

    /// Get the number of hydrated elements.
    pub fn hydrated_count(&self) -> usize {
        self.hydrated.borrow().len()
    }

    /// Get the number of pending (not yet hydrated) elements.
    pub fn pending_count(&self) -> usize {
        self.entries
            .borrow()
            .values()
            .filter(|e| e.state != ElementHydrationState::Hydrated)
            .count()
    }

    /// Get all element IDs that are pending hydration.
    pub fn pending_ids(&self) -> Vec<usize> {
        self.entries
            .borrow()
            .iter()
            .filter(|(_, e)| e.state != ElementHydrationState::Hydrated)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get all element IDs with a specific strategy.
    pub fn ids_with_strategy(&self, strategy: ElementHydrationStrategy) -> Vec<usize> {
        self.entries
            .borrow()
            .iter()
            .filter(|(_, e)| e.config.strategy == strategy)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.hydrated.borrow_mut().clear();
        self.pending_callbacks.borrow_mut().clear();
    }

    /// Generate the JavaScript to set up intersection observers for OnVisible elements.
    pub fn intersection_observer_script(&self) -> String {
        let ids = self.ids_with_strategy(ElementHydrationStrategy::OnVisible);
        if ids.is_empty() {
            return String::new();
        }

        let id_array: String = ids
            .iter()
            .map(|id| format!("'rye-el-{}'", id))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            r#"(function(){{var observer=new IntersectionObserver(function(entries){{entries.forEach(function(entry){{if(entry.isIntersecting){{var id=entry.target.id.replace('rye-el-','');ryeHydrateElement(parseInt(id));observer.unobserve(entry.target);}}}});}});[{ids}].forEach(function(id){{var el=document.getElementById(id);if(el)observer.observe(el);}});}})();"#,
            ids = id_array,
        )
    }
}

impl Default for ElementHydrationManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global element hydration manager.
thread_local! {
    static GLOBAL_MANAGER: RefCell<Option<Rc<ElementHydrationManager>>> = const { RefCell::new(None) };
}

/// Initialize the global element hydration manager.
pub fn init_global_manager() {
    GLOBAL_MANAGER.with(|m| {
        *m.borrow_mut() = Some(Rc::new(ElementHydrationManager::new()));
    });
}

/// Get the global element hydration manager.
pub fn global_manager() -> Option<Rc<ElementHydrationManager>> {
    GLOBAL_MANAGER.with(|m| m.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_immediate() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::immediate(), "Counter");
        assert_eq!(manager.state(1), Some(ElementHydrationState::Pending));
        manager.hydrate_immediate();
        assert_eq!(manager.state(1), Some(ElementHydrationState::Hydrated));
        assert!(manager.is_hydrated(1));
    }

    #[test]
    fn test_hydration_on_interaction() {
        let manager = ElementHydrationManager::new();
        manager.register(2, ElementHydrationConfig::on_interaction(), "Button");
        assert!(!manager.is_hydrated(2));
        manager.mark_interaction(2);
        assert!(manager.is_hydrated(2));
    }

    #[test]
    fn test_hydration_on_visible() {
        let manager = ElementHydrationManager::new();
        manager.register(3, ElementHydrationConfig::on_visible(), "Card");
        assert!(!manager.is_hydrated(3));
        manager.mark_visible(3);
        assert!(manager.is_hydrated(3));
    }

    #[test]
    fn test_hydration_on_idle() {
        let manager = ElementHydrationManager::new();
        manager.register(4, ElementHydrationConfig::on_idle(), "Widget");
        manager.register(5, ElementHydrationConfig::on_idle(), "Widget2");
        manager.mark_idle();
        assert!(manager.is_hydrated(4));
        assert!(manager.is_hydrated(5));
    }

    #[test]
    fn test_hydration_callback() {
        let manager = ElementHydrationManager::new();
        let called = Rc::new(RefCell::new(false));
        let called_clone = Rc::clone(&called);
        manager.register(6, ElementHydrationConfig::immediate(), "Test");
        manager.on_hydrate(6, move || {
            *called_clone.borrow_mut() = true;
        });
        manager.hydrate(6);
        assert!(*called.borrow());
    }

    #[test]
    fn test_hydration_skip_already_hydrated() {
        let manager = ElementHydrationManager::new();
        manager.register(7, ElementHydrationConfig::immediate(), "Test");
        manager.hydrate(7);
        // Hydrating again should be a no-op
        manager.hydrate(7);
        assert_eq!(manager.hydrated_count(), 1);
    }

    #[test]
    fn test_hydration_counts() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::immediate(), "A");
        manager.register(2, ElementHydrationConfig::on_idle(), "B");
        manager.register(3, ElementHydrationConfig::on_interaction(), "C");
        assert_eq!(manager.registered_count(), 3);
        assert_eq!(manager.pending_count(), 3);
        assert_eq!(manager.hydrated_count(), 0);
        manager.hydrate(1);
        assert_eq!(manager.pending_count(), 2);
        assert_eq!(manager.hydrated_count(), 1);
    }

    #[test]
    fn test_hydration_pending_ids() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::immediate(), "A");
        manager.register(2, ElementHydrationConfig::immediate(), "B");
        manager.hydrate(1);
        let pending = manager.pending_ids();
        assert!(pending.contains(&2));
        assert!(!pending.contains(&1));
    }

    #[test]
    fn test_ids_with_strategy() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::on_visible(), "A");
        manager.register(2, ElementHydrationConfig::on_interaction(), "B");
        manager.register(3, ElementHydrationConfig::on_visible(), "C");
        let visible_ids = manager.ids_with_strategy(ElementHydrationStrategy::OnVisible);
        assert_eq!(visible_ids.len(), 2);
        assert!(visible_ids.contains(&1));
        assert!(visible_ids.contains(&3));
    }

    #[test]
    fn test_hydrate_all() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::on_interaction(), "A");
        manager.register(2, ElementHydrationConfig::on_idle(), "B");
        manager.hydrate_all();
        assert_eq!(manager.hydrated_count(), 2);
    }

    #[test]
    fn test_clear() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::immediate(), "A");
        manager.hydrate(1);
        manager.clear();
        assert_eq!(manager.registered_count(), 0);
        assert_eq!(manager.hydrated_count(), 0);
    }

    #[test]
    fn test_delayed_config() {
        let config = ElementHydrationConfig::delayed(500);
        assert_eq!(config.strategy, ElementHydrationStrategy::Delayed);
        assert_eq!(config.delay_ms, 500);
    }

    #[test]
    fn test_intersection_observer_script() {
        let manager = ElementHydrationManager::new();
        manager.register(1, ElementHydrationConfig::on_visible(), "A");
        manager.register(2, ElementHydrationConfig::on_visible(), "B");
        let script = manager.intersection_observer_script();
        assert!(script.contains("IntersectionObserver"));
        assert!(script.contains("rye-el-1"));
        assert!(script.contains("rye-el-2"));
    }

    #[test]
    fn test_intersection_observer_script_empty() {
        let manager = ElementHydrationManager::new();
        let script = manager.intersection_observer_script();
        assert!(script.is_empty());
    }
}
