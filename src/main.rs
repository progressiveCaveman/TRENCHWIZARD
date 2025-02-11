use engine::{Engine, EngineDelegate};
use pixels::Error;

use winit::event_loop::EventLoop;

fn main() -> Result<(), Error> {
    let delegate = VillageModeDelegate;

    let event_loop = EventLoop::new();
    let mut engine = Engine::new(&event_loop, Box::new(delegate));

    // let mut last_time = Instant::now();
    // main event loop
    event_loop.run(move |event, _, control_flow| {
        engine.run(event, control_flow);
    });
}

struct VillageModeDelegate;

impl EngineDelegate for VillageModeDelegate {
    fn update(&mut self) {
        // println!("update");
    }

    fn time_advanced(&mut self) {
        println!("time_advanced");
    }
}