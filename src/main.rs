use std::time::Instant;

use engine::game_modes::{get_settings, GameMode};

use error_iter::ErrorIter as _;
use game::{Game, GameState};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};

use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod assets;
pub mod input_handler;
pub mod screen;
pub mod game;

const WIDTH: i32 = 1560;
const HEIGHT: i32 = 960;

pub const DISABLE_MAPGEN_ANIMATION: bool = false;

pub const MAIN_MENU_OPTIONS: usize = 2;
pub const MODE_SELECT_OPTIONS: usize = 3;

fn main() -> Result<(), Error> {
    env_logger::init();

    // create the window
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // init pixels frame buffer with window
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut game = Game::new();
    game.world_sim.reset_engine(get_settings(GameMode::RL));
    // game.engine.get_log_mut().messages.push("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string());
    game.screen.setup_consoles();
    game.set_state(GameState::PreTurn);

    let mut last_time = Instant::now();
    // main event loop
    event_loop.run(move |event, _, control_flow| {

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            game.draw(pixels.frame_mut(), input.mouse_pressed(0));
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }  

            game.frame_time = last_time.elapsed().as_millis() as i32;
            last_time = Instant::now();

            // query the change in mouse this update
            if input.mouse_diff() != (0.0, 0.0) {
                game.screen.mouse_pos = (input.mouse().unwrap().0 as i32, input.mouse().unwrap().1 as i32);
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            game.update();
            window.request_redraw();
        }

        // this should probably come before update
        match event {
            Event::WindowEvent { event, .. } => {
                let state = input_handler::handle_input(event, &mut game);
                if state != GameState::None {
                    game.set_state(state);
                }
            },
            _ => {}
        }

        if game.state == GameState::Exit {
            *control_flow = ControlFlow::Exit;
            return;
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
