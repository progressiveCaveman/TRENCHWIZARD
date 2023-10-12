#[macro_use]
extern crate lazy_static;

use components::{Equipped, InBackpack, Player, Position, Vision, PlayerID, GameLog, FrameTime, PPoint, Turn, RNG};
use game_modes::{GameSettings, GameMode};
use map::Map;

use rltk::Point;
use shipyard::{
    EntitiesView, EntityId, Get, UniqueView, UniqueViewMut, View, ViewMut, World, AllStoragesViewMut,
};

pub mod components;
pub mod map;
pub mod utils;
pub mod map_builders;
pub mod entity_factory;
pub mod colors;
pub mod worldgen;
pub mod game_modes;
pub mod tiles;

pub const SHOW_MAPGEN_ANIMATION: bool = true;
pub const MAPGEN_FRAME_TIME: f32 = 25.0;

pub const TILE_SIZE: usize = 10;
pub const SCALE: f32 = 1.0;

pub const OFFSET_X: usize = 31;
pub const OFFSET_Y: usize = 11;

pub const DISABLE_AI: bool = false;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items,
    NPC,
    Player,
    Particle,
}

pub struct Engine {
    pub world: World,
    pub settings: GameSettings,
    pub first_run: bool,
}

impl Engine {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            world: World::new(),
            first_run: false,
            settings,
        }
    }

    pub fn get_log(&self) -> UniqueView<'_, GameLog> {
        self.world.borrow::<UniqueView<GameLog>>().unwrap()
    }

    pub fn get_log_mut(&self) -> UniqueViewMut<'_, GameLog> {
        self.world.borrow::<UniqueViewMut<GameLog>>().unwrap()
    }

    pub fn get_map(&self) -> UniqueView<'_, Map> {
        self.world.borrow::<UniqueView<Map>>().unwrap()
    }

    pub fn get_player_id(&self) -> UniqueView<'_, PlayerID> {
        self.world.borrow::<UniqueView<PlayerID>>().unwrap()
    }

    pub fn run_systems(world: &mut World, _player_turn: bool, ai_turn: bool) {
        // // if player_turn {
        // world.run(system_fire::run_fire_system);
        // // }
        // world.run(system_visibility::run_visibility_system);

        // world.run(effects::run_effects_queue);

        // if ai_turn && !DISABLE_AI {
        //     world.run(system_pathfinding::run_pathfinding_system);
        //     world.run(system_ai_fish::run_fish_ai);
        //     world.run(system_ai::run_ai_system);
        // }

        // world.run(effects::run_effects_queue);

        // world.run(system_map_indexing::run_map_indexing_system);

        // world.run(system_melee_combat::run_melee_combat_system);
        // world.run(item_system::run_inventory_system);
        // world.run(system_dissasemble::run_dissasemble_system);
        // world.run(item_system::run_drop_item_system);
        // world.run(item_system::run_unequip_item_system);
        // world.run(item_system::run_item_use_system);
        // world.run(system_particle::spawn_particles);

        // world.run(effects::run_effects_queue);
        // world.run(system_map_indexing::run_map_indexing_system);
    }

    pub fn entities_to_delete_on_level_change(world: &mut World) -> Vec<EntityId> {
        let mut ids_to_delete: Vec<EntityId> = Vec::new();

        let entities = world.borrow::<EntitiesView>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

        let vplayer = world.borrow::<View<Player>>().unwrap();
        let vpack = world.borrow::<View<InBackpack>>().unwrap();
        let vequipped = world.borrow::<View<Equipped>>().unwrap();

        for id in entities.iter() {
            let mut to_delete = true;

            if let Ok(_) = vplayer.get(id) {
                to_delete = false;
            } else if let Ok(backpack) = vpack.get(id) {
                if backpack.owner == player_id {
                    to_delete = false;
                }
            } else if let Ok(equipped) = vequipped.get(id) {
                if equipped.owner == player_id {
                    to_delete = false;
                }
            }

            if to_delete {
                ids_to_delete.push(id);
            }
        }

        ids_to_delete
    }

    pub fn generate_map(world: &mut World, new_depth: usize) {
        // delete all entities
        let ids_to_delete = Self::entities_to_delete_on_level_change(world);
        for id in ids_to_delete {
            world.delete_entity(id);
        }

        // self.mapgen_data.index = 0;
        // self.mapgen_data.timer = 0.0;
        // self.mapgen_data.history.clear();

        // get game mode
        let settings = *world.borrow::<UniqueView<GameSettings>>().unwrap();

        // Generate map
        // TODO eventually this should not look at mode, but use map vonfig info from settings
        let mut map_builder = match settings.mode {
            GameMode::VillageSim => map_builders::village_builder(new_depth, settings.mapsize),
            GameMode::RL => map_builders::rl_builder(new_depth, settings.mapsize),
            GameMode::OrcHalls => map_builders::orc_halls_builder(new_depth, settings.mapsize),
        };

        map_builder.build_map();

        // self.mapgen_data.history = map_builder.get_map_history();

        let start_pos;
        {
            let mut map = world.borrow::<UniqueViewMut<Map>>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder.get_starting_position().ps.first().unwrap().clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(world);

        // Update player position
        world.run(
            |mut ppos: UniqueViewMut<PPoint>,
             player_id: UniqueView<PlayerID>,
             mut vpos: ViewMut<Position>,
             mut vvs: ViewMut<Vision>| {
                *ppos = PPoint(Point::new(start_pos.x, start_pos.y));
                if let Ok(pos) = (&mut vpos).get(player_id.0) {
                    pos.ps[0] = ppos.0;
                }

                if let Ok(vs) = (&mut vvs).get(player_id.0) {
                    vs.dirty = true;
                }
            },
        );
    }

    // pub fn next_level(world: &mut World) {
    //     // Generate new map
    //     let current_depth;
    //     {
    //         let map = world.borrow::<UniqueViewMut<Map>>().unwrap();
    //         current_depth = map.depth;
    //     }
    //     Self::generate_map(world, current_depth + 1);

    //     // Notify player
    //     let mut log = world.borrow::<UniqueViewMut<GameLog>>().unwrap();
    //     log.messages.push("You descend in the staircase".to_string());
    // }

    pub fn reset_engine(&mut self, settings: GameSettings) {
        // Delete everything
        // world.clear();
        self.world = World::new();

        // Re-add defaults for all uniques
        self.world.add_unique(Map::new(settings.mapsize));
        self.world.add_unique(PPoint(Point::new(0, 0)));
        self.world.add_unique(Turn(0));
        self.world.add_unique(RNG(rltk::RandomNumberGenerator::new()));

        let player_id = self
            .world
            .run(|mut store: AllStoragesViewMut| entity_factory::player(&mut store, (0, 0)));
        self.world.add_unique(PlayerID(player_id));

        self.world.add_unique(settings);
        self.world.add_unique(GameLog { messages: vec![] });
        // self.world.add_unique(system_particle::ParticleBuilder::new());
        self.world.add_unique(FrameTime(0.));

        match settings.mode {
            GameMode::VillageSim => {
                // self.world.add_component(player_id, IsCamera {});
            }
            _ => {}
        }

        // Generate new map
        Self::generate_map(&mut self.world, 1);
    }
}
