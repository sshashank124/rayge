mod app;
mod input;
mod time_stepper;

use tracing::Level;
use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};
use winit::event_loop::{ControlFlow, EventLoop};

use app::App;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_span_events(FmtSpan::ACTIVE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("failed to set default subscriber");

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("failed to run app");
    if let Err(e) = app.close() {
        eprintln!("error occured while running: {e}");
    }
}
