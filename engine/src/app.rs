use std::error::Error;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

use renderer::Renderer;

use crate::input;

mod conf {
    pub const WINDOW_TITLE: &str = "Engine";
    pub const FRAME_RESOLUTION: (u32, u32) = (640, 480);
}

#[derive(Default)]
pub struct App {
    // state
    inputs: input::State,
    error: Option<Box<dyn Error>>,
    // graphics
    window_attributes: WindowAttributes,
    graphics: Option<Graphics>,
}

struct Graphics {
    renderer: Renderer,
    needs_resizing: bool,
    window: Window,
}

impl App {
    pub fn new() -> Self {
        let window_attributes = Window::default_attributes()
            .with_inner_size(LogicalSize::<u32>::from(conf::FRAME_RESOLUTION))
            .with_title(conf::WINDOW_TITLE);

        Self {
            window_attributes,
            ..Default::default()
        }
    }

    pub fn close(self) -> Result<(), Box<dyn Error>> {
        self.error.map_or(Ok(()), Err)
    }
    // fn render(&mut self) {}
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            event_loop.set_control_flow(ControlFlow::Poll);

            self.graphics = Some(
                match event_loop.create_window(self.window_attributes.clone()) {
                    Err(e) => {
                        self.error = Some(e.into());
                        event_loop.exit();
                        return;
                    }
                    Ok(window) => {
                        let renderer = match Renderer::new(&window) {
                            Err(e) => {
                                self.error = Some(e.into());
                                event_loop.exit();
                                return;
                            }
                            Ok(renderer) => renderer,
                        };
                        Graphics {
                            renderer,
                            needs_resizing: false,
                            window,
                        }
                    }
                },
            );
        }
    }

    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // WindowEvent::RedrawRequested => self.render(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(graphics) = &mut self.graphics {
                    graphics.needs_resizing = true;
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                self.inputs.handle_key(key_code, state);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.inputs.handle_mouse_button(button, state);
            }
            _ => (),
        }
    }
}
