use assets::Assets;
use engine::game_modes::{get_settings, GameMode};

use engine::Engine;
use error_iter::ErrorIter as _;
use input_handler::{handle_input, Action};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};

use screen::{Screen, MAX_ZOOM};
use screen::console::ConsoleMode;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod assets;
pub mod input_handler;
pub mod screen;

const SCALE: usize = 2;
const WIDTH: usize = 640 * SCALE;
const HEIGHT: usize = 480 * SCALE;

type Image = (Vec<[u8; 4]>, (usize, usize));

pub struct Game {
    pub engine: Engine,
    pub screen: Screen,
    pub assets: Assets,
    pub tick: usize,
    pub state: GameState,
    pub history_timer: usize,
    pub history_step: usize
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    Waiting,
    MainMenu,
    ShowMapHistory
}

impl Game {
    fn new() -> Self {
        Self {
            engine: Engine::new(get_settings(GameMode::RL)),
            screen: Screen::new((WIDTH, HEIGHT)),
            assets: Assets::new(),
            tick: 0,
            state: GameState::MainMenu,
            history_timer: 0,
            history_step: 0,
        }
    }

    /// Update the game state
    fn update(&mut self) {
        self.tick += 1;
        if self.tick % 100 == 0 {
            self.engine.get_log_mut().messages.push(format!("Test {}", self.tick / 100));
        }

        {
            let map = self.engine.get_map();

            // automatically zoom in on small maps
            for c in self.screen.consoles.iter_mut() {
                if c.mode == ConsoleMode::WorldMap {
                    while c.zoom < MAX_ZOOM && (c.zoom + 1) * map.size.0 < c.size.0 && (c.zoom + 1) * map.size.1 < c.size.1 {
                        c.zoom += 1;
                    }
                }
            }
        }

        // Main loop
        match self.state {
            GameState::Waiting => {
                self.engine.run_systems();
            },
            GameState::MainMenu => {

            },
            GameState::ShowMapHistory => {
                self.history_timer += 1;
                self.history_step = self.history_timer / 10;
                let map = self.engine.get_map();
                
                if self.history_step > map.history.len() {
                    self.state = GameState::Waiting;
                }
            },
        }
    }

    /// Draw the `World` state to the frame buffer.
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        self.screen.draw(frame, &self);
    }

    pub fn set_state(&mut self, state: GameState) {
        match state {
            GameState::ShowMapHistory => self.history_timer = 0,
            _ => {}
        }

        self.state = state;
    }
}

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
    game.engine.reset_engine(get_settings(GameMode::OrcHalls));
    game.engine.get_log_mut().messages.push("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string());
    game.screen.setup_consoles();

    // main event loop
    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            game.draw(pixels.frame_mut());
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

            match handle_input(&input, &mut game) {
                Action::None => {}
                Action::Exit => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
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
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
