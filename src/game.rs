use engine::{Engine, game_modes::{get_settings, GameMode}, components::FrameTime, systems::system_particle, effects, utils::InvalidPoint};
use shipyard::{EntityId, UniqueViewMut};

use crate::{screen::{Screen, menu_config::{MainMenuSelection, ModeSelectSelection}, console::ConsoleMode}, assets::Assets, WIDTH, HEIGHT, DISABLE_MAPGEN_ANIMATION};


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
    pub fn new() -> Self {
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
    pub fn update(&mut self) {
        self.tick += 1;

        // automatically zoom in on small maps
        self.screen.autozoomn_world_map(&self.engine.get_map());

        // update frame time for particle engine
        self.engine.world.borrow::<UniqueViewMut<FrameTime>>().unwrap().0 = self.frame_time as f32;

        // let mut new_runstate = self.state;
        // let player_id = self.engine.get_player_id();
        
        self.engine.world.run(system_particle::update_particles);
        self.engine.world.run(effects::run_effects_queue);

        // update map console to follow player if applicable
        if self.engine.settings.follow_player {
            for c in self.screen.consoles.iter_mut() {
                if c.mode == ConsoleMode::WorldMap {
                    let ppos = self.engine.get_player_pos().0.to_xy();

                    let mp = (ppos.0 - c.size.0 / (2 * c.tile_size), ppos.1 - c.size.1 / (2 * c.tile_size));
                    c.map_pos = (i32::max(0, mp.0), i32::max(0, mp.1))
                }
            }
        }

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
    pub fn draw(&self, frame: &mut [u8]) {
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