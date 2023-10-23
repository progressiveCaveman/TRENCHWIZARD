use std::time::Instant;

use assets::Assets;
use engine::components::FrameTime;
use engine::game_modes::{get_settings, GameMode};

use engine::{Engine, effects};
use engine::systems::system_particle;
use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};

use screen::menu_config::{MainMenuSelection, ModeSelectSelection};
use screen::Screen;
use shipyard::{EntityId, UniqueViewMut};
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod assets;
pub mod input_handler;
pub mod screen;

const SCALE: i32 = 2;
const WIDTH: i32 = 640 * SCALE;
const HEIGHT: i32 = 480 * SCALE;

pub const DISABLE_AI: bool = false;
pub const DISABLE_MAPGEN_ANIMATION: bool = true;

pub const MAIN_MENU_OPTIONS: usize = 2;
pub const MODE_SELECT_OPTIONS: usize = 3;

pub struct Game {
    pub engine: Engine,
    pub screen: Screen,
    pub assets: Assets,
    pub tick: usize,
    pub state: GameState,
    pub history_timer: usize,
    pub history_step: usize,
    pub autorun: bool,
    pub frame_time: i32,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GameState {
    // returning none means game state doesn't change
    None,

    //main loop
    Waiting,
    PlayerTurn,
    AiTurn,

    MainMenu{ selection: MainMenuSelection },
    ModeSelect{ selection: ModeSelectSelection },
    ShowMapHistory,
    ShowInventory,
    ShowItemActions {
        item: EntityId,
    },
    ShowTargeting {
        range: i32,
        item: EntityId,
    },
    GameOver,
    Exit
}

impl Game {
    fn new() -> Self {
        Self {
            engine: Engine::new(get_settings(GameMode::RL)),
            screen: Screen::new((WIDTH, HEIGHT)),
            assets: Assets::new(),
            tick: 0,
            state: GameState::MainMenu { selection: MainMenuSelection::Play },
            history_timer: 0,
            history_step: 0,
            autorun: false,
            frame_time: 0,
        }
    }

    /// Update the game state
    fn update(&mut self) {
        self.tick += 1;

        // automatically zoom in on small maps
        self.screen.autozoomn_world_map(&self.engine.get_map());

        // update frame time for particle engine
        self.engine.world.borrow::<UniqueViewMut<FrameTime>>().unwrap().0 = self.frame_time as f32;

        // let mut new_runstate = self.state;
        // let player_id = self.engine.get_player_id();
        
        self.engine.world.run(system_particle::update_particles);
        self.engine.world.run(effects::run_effects_queue);

        // Main loop
        match self.state {
            GameState::Waiting => {
                if self.engine.settings.mode == GameMode::MapDemo {
                    self.engine.reset_engine(self.engine.settings);
                    self.set_state(GameState::ShowMapHistory);
                }

                if self.autorun {
                    self.set_state(GameState::PlayerTurn);

                }
            },
            GameState::PlayerTurn => {
                self.set_state(GameState::AiTurn);
                self.engine.run_systems();
            },
            GameState::AiTurn => {
                self.set_state(GameState::Waiting);
            },
            // GameState::Play => {
            //     self.engine.run_systems();

            // },
            GameState::ShowMapHistory => {
                self.history_timer += 1;
                self.history_step = self.history_timer / 5;
                let map = self.engine.get_map();
                
                if self.history_step > map.history.len() + 20 || DISABLE_MAPGEN_ANIMATION {
                    self.state = GameState::Waiting;
                }
            },
            _ => {},
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
            GameState::MainMenu { selection: _ } => {
            }
            GameState::ModeSelect { selection: _ } => {
            }
            _ => {},
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
    game.engine.reset_engine(get_settings(GameMode::RL));
    game.engine.get_log_mut().messages.push("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string());
    game.screen.setup_consoles();

    let mut last_time = Instant::now();
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

            game.frame_time = last_time.elapsed().as_millis() as i32;
            last_time = Instant::now();

            // match handle_input(&event, &mut game) {
            //     Action::None => {}
            //     Action::Exit => {
            //         *control_flow = ControlFlow::Exit;
            //         return;
            //     }
            // }    

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
