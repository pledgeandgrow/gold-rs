//! Mobile lifecycle — handle app foreground/background/suspend.

/// Mobile app lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileLifecycle {
    /// App is in foreground and active.
    Active,
    /// App is in background but still running.
    Inactive,
    /// App is suspended (may be killed at any time).
    Background,
}

/// Handle mobile lifecycle events.
pub trait MobileLifecycleHandler: 'static {
    /// Called when the app enters the foreground.
    fn on_foreground(&mut self) {}
    /// Called when the app enters the background.
    fn on_background(&mut self) {}
    /// Called when the app is about to be terminated.
    fn on_terminate(&mut self) {}
    /// Called on memory warning (release caches, etc.).
    fn on_memory_warning(&mut self) {}
}
