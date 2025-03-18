mod app;
mod input;

use winit::event_loop::EventLoop;

use app::App;

fn main() -> () {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("failed to run app");
    if let Err(e) = app.close() {
        eprintln!("error occured while running: {e}");
    }
}
