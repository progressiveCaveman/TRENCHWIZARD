use crate::{components::{FrameTime, PhysicalStats, WantsToUseItem}, simulator::effects, game_modes::{get_settings, GameMode, GameSettings}, map::XY, simulator::systems::system_particle, ui::{assets::Assets, screen::{console::ConsoleMode, menu_config::{MainMenuSelection, ModeSelectSelection}, RangedTargetResult, Screen}}, utils::InvalidPoint, world_sim::WorldSim, DISABLE_MAPGEN_ANIMATION, HEIGHT, WIDTH};
use shipyard::{EntityId, Get, UniqueViewMut, View};

pub struct Game {
    pub world_sim: WorldSim,
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
    PreTurn, // before anything happens
    PlayerActed, // after player has acted
    PostTurn, // after systems have acted

    MainMenu{ selection: MainMenuSelection },
    ModeSelect{ selection: ModeSelectSelection },
    ShowMapHistory,
    ShowInventory{ selection: usize },
    ShowItemActions {
        item: EntityId,
    },
    ShowTargeting {
        range: i32,
        item: EntityId,
        target: XY,
    },
    GameOver,
    Exit
}

impl Game {
    pub fn new() -> Self {
        Self {
            world_sim: WorldSim::new(get_settings(GameMode::RL)),
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
        self.screen.autozoom_world_map(&self.world_sim.get_map());

        // update frame time for particle engine
        self.world_sim.world.borrow::<UniqueViewMut<FrameTime>>().unwrap().0 = self.frame_time as f32;
        
        self.world_sim.world.run(system_particle::update_particles);
        self.world_sim.world.run(effects::run_effects_queue);

        // update map console to follow player if applicable
        if self.world_sim.settings.follow_player {
            for c in self.screen.consoles.iter_mut() {
                if c.mode == ConsoleMode::WorldMap {
                    let ppos = self.world_sim.get_player_pos().0.to_xy();

                    let mp = (ppos.0 - c.size.0 / (2 * c.gsize), ppos.1 - c.size.1 / (2 * c.gsize));
                    c.map_pos = (i32::max(0, mp.0), i32::max(0, mp.1))
                } else {
                    c.map_pos = (0,0);
                }
            }
        }

        // Main loop
        match self.state {
            GameState::PreTurn => {
                // check if player is dead
                let player_id = self.world_sim.get_player_id().0;
                let mut gameover = false;
                {
                    let vstats = self.world_sim.world.borrow::<View<PhysicalStats>>().unwrap();
                    if let Ok(stats) = vstats.get(player_id) {
                        if stats.hp <=0 {
                            gameover = true;
                            // dbg!("game over");
                        }
                    }
                }

                if gameover {
                    self.set_state(GameState::GameOver);
                }

                if self.autorun {
                    self.set_state(GameState::PlayerActed);
                }
            },
            GameState::PlayerActed => {
                self.set_state(GameState::PostTurn);
                self.world_sim.run_systems();
            },
            GameState::PostTurn => {
                self.set_state(GameState::PreTurn);
            },
            GameState::ShowMapHistory => {
                self.history_timer += 1;
                self.history_step = self.history_timer / 5;
                let map = self.world_sim.get_map();

                if self.history_step > map.history.len() + 20 || (DISABLE_MAPGEN_ANIMATION && self.world_sim.settings.mode != GameMode::MapDemo) {
                    self.state = GameState::PreTurn;
                }
            },
            GameState::GameOver => {
                self.reset(None);
            }
            _ => {},
        }
    }

    /// Draw the `World` state to the frame buffer.
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&mut self, frame: &mut [u8], mouseclick: bool) {
        self.screen.draw(frame, &self);

        match self.state {
            GameState::ShowTargeting { range, item, target } => {
                // self.screen.ranged_target(frame, assets, game, range, clicked)
                let (result, target) = self.screen.ranged_target(frame, &self.assets, &mut self.world_sim.world, range, mouseclick, target);
                match result {
                    RangedTargetResult::Cancel => self.state = GameState::PreTurn,
                    RangedTargetResult::NoResponse => {} ,
                    RangedTargetResult::Selected => {
                        self.world_sim.world.add_component(item, WantsToUseItem { item, target: target });
                        self.state = GameState::PlayerActed;    
                    },
                    RangedTargetResult::NewTarget { target } => self.state = GameState::ShowTargeting { range, item, target },
                }
            },
            _ => {}
        }
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

    pub fn reset(&mut self, settings: Option<GameSettings>) {
        self.state = GameState::ShowMapHistory;
        match settings {
            Some(s) => self.world_sim.reset_engine(s),
            None => self.world_sim.reset_engine(self.world_sim.settings),
        }
    }
}