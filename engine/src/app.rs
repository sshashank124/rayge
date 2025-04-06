use std::{
    error::Error,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

use renderer::Renderer;

use crate::{
    input,
    time_stepper::{TimeStepInfo, TimeStepper},
};

mod conf {
    pub const WINDOW_TITLE: &str = "RAYGE Engine";
    pub const FRAME_RESOLUTION: (u32, u32) = (640, 480);

    pub const UPDATE_FREQUENCY: u64 = 100;
    pub const TARGET_FPS: u64 = 60;
}

pub struct App {
    inputs: input::State,
    // state
    stepper: TimeStepper<{ conf::UPDATE_FREQUENCY }>,
    graphics: Option<Graphics>,
    previous_time: Instant,
    // app
    window_attributes: WindowAttributes,
    error: Option<Box<dyn Error>>,
}

struct Graphics {
    stepper: TimeStepper<{ conf::TARGET_FPS }>,
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
            inputs: input::State::default(),
            stepper: TimeStepper::default(),
            graphics: None,
            previous_time: Instant::now(),
            window_attributes,
            error: None,
        }
    }

    pub fn close(self) -> Result<(), Box<dyn Error>> {
        self.error.map_or(Ok(()), Err)
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let now = Instant::now();
        let delta_time = now.duration_since(self.previous_time);
        self.previous_time = now;

        self.stepper += delta_time;
        for TimeStepInfo { now: _, delta: _ } in &mut self.stepper {
            // update state
        }

        if let Some(graphics) = &mut self.graphics {
            graphics.update(delta_time)?;
        }

        Ok(())
    }
}

impl Graphics {
    fn new(renderer: Renderer, window: Window) -> Self {
        Self {
            stepper: TimeStepper::default(),
            renderer,
            needs_resizing: false,
            window,
        }
    }

    fn update(&mut self, delta_time: Duration) -> Result<(), renderer::Error> {
        self.stepper += delta_time;
        if !self.needs_resizing || self.renderer.resize()? {
            self.needs_resizing = false;
            for TimeStepInfo { now: _, delta: _ } in &mut self.stepper {
                self.needs_resizing = self.renderer.render()?;
            }
        }
        Ok(())
    }
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
                        Graphics::new(renderer, window)
                    }
                },
            );
        }
    }

    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.update() {
            self.error = Some(e);
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
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
