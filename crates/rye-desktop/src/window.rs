//! Window — desktop window management via winit.

/// A desktop window.
pub struct Window {
    // TODO: winit window, event loop handle
}

impl Window {
    /// Create a new window with default settings.
    pub fn new() -> Self {
        // TODO: create winit window
        Self {}
    }

    /// Set the window title.
    pub fn set_title(&mut self, _title: &str) {
        // TODO
    }

    /// Set the window size.
    pub fn set_size(&mut self, _width: u32, _height: u32) {
        // TODO
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
