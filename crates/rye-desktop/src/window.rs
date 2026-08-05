//! Window — desktop window management and event loop via winit.
//!
//! Provides a `run` function that creates a winit window, initializes the
//! GPU renderer, and runs the event loop. The caller provides a render
//! callback that is invoked each frame to update the render tree.

use std::sync::Arc;
use std::sync::Mutex;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowAttributes, WindowId};

use crate::input::{InputEvent, KeyCode};
use crate::native_renderer::NativeRenderer;

/// Configuration for a desktop window.
pub struct WindowConfig {
    /// Window title.
    pub title: String,
    /// Initial width in logical pixels.
    pub width: f64,
    /// Initial height in logical pixels.
    pub height: f64,
    /// Whether the window is resizable.
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "rye app".to_string(),
            width: 800.0,
            height: 600.0,
            resizable: true,
        }
    }
}

/// Application state held by the winit event loop.
struct App {
    config: WindowConfig,
    renderer: Option<NativeRenderer>,
    window: Option<winit::window::Window>,
    cursor_pos: winit::dpi::PhysicalPosition<f64>,
    render_callback: Arc<Mutex<Box<dyn FnMut(&mut NativeRenderer) + Send>>>,
    input_callback: Arc<Mutex<Box<dyn FnMut(&InputEvent) + Send>>>,
}

/// Run the desktop application with a render callback.
///
/// The render callback is called every frame with the renderer, allowing
/// the caller to update the render tree. The input callback is called
/// for each input event.
///
/// This function blocks until the window is closed.
pub fn run(
    config: WindowConfig,
    render_callback: impl FnMut(&mut NativeRenderer) + Send + 'static,
    input_callback: impl FnMut(&InputEvent) + Send + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        config,
        renderer: None,
        window: None,
        cursor_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        render_callback: Arc::new(Mutex::new(Box::new(render_callback))),
        input_callback: Arc::new(Mutex::new(Box::new(input_callback))),
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable);

        let window = event_loop
            .create_window(attrs)
            .expect("Failed to create window");

        let mut renderer = NativeRenderer::new();
        renderer.attach_gpu(&window);

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                {
                    let mut cb = self.render_callback.lock().unwrap();
                    cb(renderer);
                }
                renderer.render_frame();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
                let evt = InputEvent::MouseMove {
                    x: position.x,
                    y: position.y,
                };
                let mut cb = self.input_callback.lock().unwrap();
                cb(&evt);
            }

            WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                ..
            } => {
                let evt = InputEvent::Click {
                    x: self.cursor_pos.x,
                    y: self.cursor_pos.y,
                };
                let mut cb = self.input_callback.lock().unwrap();
                cb(&evt);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;
                use winit::keyboard::{Key, NamedKey};
                let key = match &event.logical_key {
                    Key::Named(NamedKey::Enter) => KeyCode::Enter,
                    Key::Named(NamedKey::Escape) => KeyCode::Escape,
                    Key::Named(NamedKey::Tab) => KeyCode::Tab,
                    Key::Named(NamedKey::Space) => KeyCode::Space,
                    Key::Named(NamedKey::Backspace) => KeyCode::Backspace,
                    Key::Named(NamedKey::ArrowUp) => KeyCode::ArrowUp,
                    Key::Named(NamedKey::ArrowDown) => KeyCode::ArrowDown,
                    Key::Named(NamedKey::ArrowLeft) => KeyCode::ArrowLeft,
                    Key::Named(NamedKey::ArrowRight) => KeyCode::ArrowRight,
                    Key::Character(s) => {
                        let c = s.as_str().chars().next().unwrap_or('\0');
                        KeyCode::Char(c)
                    }
                    _ => KeyCode::Char('\0'),
                };
                let evt = match event.state {
                    ElementState::Pressed => InputEvent::KeyPress { key },
                    ElementState::Released => InputEvent::KeyRelease { key },
                };
                let mut cb = self.input_callback.lock().unwrap();
                cb(&evt);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (x as f64 * 20.0, y as f64 * 20.0)
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => (pos.x, pos.y),
                };
                let evt = InputEvent::Scroll { dx, dy };
                let mut cb = self.input_callback.lock().unwrap();
                cb(&evt);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
