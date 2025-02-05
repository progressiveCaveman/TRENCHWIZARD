use engine::Engine;
use pixels::Error;

use winit::event_loop::EventLoop;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut engine = Engine::new(&event_loop);

    // let mut last_time = Instant::now();
    // main event loop
    event_loop.run(move |event, _, control_flow| {
        engine.run(event, control_flow);
    });
}
