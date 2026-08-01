//! Hydration — attach signal subscriptions to server-rendered DOM.

/// Hydrate server-rendered HTML by reading `data-rye-*` markers
/// and attaching event listeners + signal subscriptions.
pub fn hydrate() {
    // TODO: walk DOM, find data-rye-id and data-rye-signal attributes,
    // match them to component tree, attach listeners
}
