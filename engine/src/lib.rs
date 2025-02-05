use error_iter::ErrorIter as _;

use game::{Game, GameState};
use game_modes::{get_settings, GameMode};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use ui::input_handler;
use winit::{dpi::LogicalSize, event::Event, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};
use winit_input_helper::WinitInputHelper;


#[macro_use]
extern crate lazy_static;

pub mod map;
pub mod utils;
pub mod entity_factory;
pub mod game_modes;
pub mod tiles;
pub mod ai;
pub mod player;
pub mod world_sim;
pub mod game;
pub mod ui;
pub mod generators;
pub mod simulator;


pub const SHOW_MAPGEN_ANIMATION: bool = true;
pub const MAPGEN_FRAME_TIME: f32 = 25.0;

pub const TILE_SIZE: usize = 10;
pub const SCALE: f32 = 1.0;

pub const OFFSET_X: usize = 31;
pub const OFFSET_Y: usize = 11;

pub const DISABLE_AI: bool = false;
pub const DISABLE_FOV: bool = true;

//settings from main, if that matters
const WIDTH: i32 = 1560;
const HEIGHT: i32 = 960;

pub const DISABLE_MAPGEN_ANIMATION: bool = false;

pub const MAIN_MENU_OPTIONS: usize = 2;
pub const MODE_SELECT_OPTIONS: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items = 0,
    NPC,
    Player,
    Particle,
}

pub struct Engine {
    input: WinitInputHelper,
    window: Window,
    pixels: Pixels,

    pub game: Game,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        env_logger::init();

        // create the window
        // let event_loop = EventLoop::new();
        let input = WinitInputHelper::new();
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
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap() //todo error check unwrap
        };
    
        let mut game = Game::new();
        game.world_sim.reset_engine(get_settings(GameMode::RL));
        // game.engine.get_log_mut().messages.push("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string());
        game.screen.setup_consoles();
        game.set_state(GameState::PreTurn);

        Self {
            input,
            window,
            pixels,
            game: game,
        }
    }

    pub fn run(&mut self, event: Event<'_, ()>, control_flow: &mut ControlFlow) {

        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            self.game.draw(self.pixels.frame_mut(), self.input.mouse_pressed(0));
            if let Err(err) = self.pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if self.input.update(&event) {
            // Close events
            if self.input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }  

            // self.game.frame_time = last_time.elapsed().as_millis() as i32;
            // last_time = Instant::now();

            // query the change in mouse this update
            if self.input.mouse_diff() != (0.0, 0.0) {
                self.game.screen.mouse_pos = (self.input.mouse().unwrap().0 as i32, self.input.mouse().unwrap().1 as i32);
            }

            // Resize the window
            if let Some(size) = self.input.window_resized() {
                if let Err(err) = self.pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            self.game.update();
            self.window.request_redraw();
        }

        // this should probably come before update
        match event {
            Event::WindowEvent { event, .. } => {
                let state = input_handler::handle_input(event, &mut self.game);
                if state != GameState::None {
                    self.game.set_state(state);
                }
            },
            _ => {}
        }

        if self.game.state == GameState::Exit {
            *control_flow = ControlFlow::Exit;
            return;
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
