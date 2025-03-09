mod app;
mod input;

use winit::event_loop::EventLoop;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    app.close()
}
