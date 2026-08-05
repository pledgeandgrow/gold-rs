//! Hydration — attach signal subscriptions to server-rendered DOM.
//!
//! After SSR produces HTML with `data-rye-id` markers, the client calls
//! `hydrate()` to walk the DOM, find those markers, and match them to
//! the component tree so event listeners and signal subscriptions can be
//! attached without re-rendering the entire tree.

/// A hydration target found in the DOM.
#[derive(Debug, Clone)]
pub struct HydrationTarget {
    /// The `data-rye-id` value (e.g. "r0", "r1").
    pub id: String,
    /// Event names that need listeners (from `data-rye-event` attributes).
    pub events: Vec<String>,
}

/// Hydrate server-rendered HTML by reading `data-rye-*` markers.
///
/// Walks the DOM looking for elements with `data-rye-id` attributes.
/// Returns a list of hydration targets that can be matched against
/// the client-side component tree.
///
/// In a WASM environment, this calls into `web-sys` to walk the DOM.
/// In non-WASM environments, it returns an empty list (for testing).
#[cfg(target_arch = "wasm32")]
pub fn hydrate() -> Vec<HydrationTarget> {
    use wasm_bindgen::JsCast;

    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(doc) => doc,
        None => return Vec::new(),
    };

    let mut targets = Vec::new();

    // Query all elements with data-rye-id
    if let Ok(elements) = document.query_selector_all("[data-rye-id]") {
        let length = elements.length();
        for i in 0..length {
            if let Some(node) = elements.item(i) {
                if let Ok(el) = node.dyn_into::<web_sys::Element>() {
                    let id = el.get_attribute("data-rye-id").unwrap_or_default();
                    let events: Vec<String> = el
                        .get_attribute("data-rye-event")
                        .map(|e| vec![e])
                        .unwrap_or_default();

                    targets.push(HydrationTarget { id, events });
                }
            }
        }
    }

    targets
}

/// Non-WASM stub for hydration.
#[cfg(not(target_arch = "wasm32"))]
pub fn hydrate() -> Vec<HydrationTarget> {
    Vec::new()
}
