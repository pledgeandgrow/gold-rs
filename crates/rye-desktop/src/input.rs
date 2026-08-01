//! Input — cross-platform input handling for native renderer.

/// A unified input event from the native platform.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse click.
    Click { x: f64, y: f64 },
    /// Mouse move.
    MouseMove { x: f64, y: f64 },
    /// Key press.
    KeyPress { key: KeyCode },
    /// Key release.
    KeyRelease { key: KeyCode },
    /// Touch start.
    TouchStart { x: f64, y: f64, id: u32 },
    /// Touch end.
    TouchEnd { x: f64, y: f64, id: u32 },
    /// Touch move.
    TouchMove { x: f64, y: f64, id: u32 },
    /// Scroll.
    Scroll { dx: f64, dy: f64 },
    /// Resize.
    Resize { width: u32, height: u32 },
}

/// Keyboard key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// Enter key.
    Enter,
    /// Escape key.
    Escape,
    /// Tab key.
    Tab,
    /// Space key.
    Space,
    /// Backspace key.
    Backspace,
    /// Arrow up.
    ArrowUp,
    /// Arrow down.
    ArrowDown,
    /// Arrow left.
    ArrowLeft,
    /// Arrow right.
    ArrowRight,
    /// Character key.
    Char(char),
}
