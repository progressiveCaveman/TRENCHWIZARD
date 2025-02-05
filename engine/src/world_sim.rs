use crate::components::{Equipped, Player, Position, Vision, PlayerID, GameLog, FrameTime, PPoint, Turn, RNG, Inventory};
use crate::effects::{add_effect, EffectType};
use crate::game_modes::{GameSettings, GameMode};
use crate::map::Map;
use crate::systems::system_particle;
use crate::systems::system_gas;

use rltk::Point;
use shipyard::{
    EntitiesView, EntityId, Get, UniqueView, UniqueViewMut, View, ViewMut, World, AllStoragesViewMut,
};

pub struct WorldSim {
    pub world: World,
    pub settings: GameSettings,
    pub first_run: bool,
}

impl WorldSim {
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

    pub fn get_player_pos(&self) -> UniqueView<'_, PPoint> {
        self.world.borrow::<UniqueView<PPoint>>().unwrap()
    }

    pub fn run_systems(&mut self) {
        crate::systems::run_systems(&mut self.world, true, true);
    }

    pub fn entities_to_delete_on_level_change(world: &mut World) -> Vec<EntityId> {
        let mut ids_to_delete: Vec<EntityId> = Vec::new();

        let entities = world.borrow::<EntitiesView>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

        let vplayer = world.borrow::<View<Player>>().unwrap();
        let vinv = world.borrow::<View<Inventory>>().unwrap();
        let vequipped = world.borrow::<View<Equipped>>().unwrap();

        for id in entities.iter() {
            let mut to_delete = true;

            if let Ok(_) = vplayer.get(id) {
                to_delete = false;
            } else if let Ok(inventory) = vinv.get(player_id) {
                for e in inventory.items.iter() {
                    if *e == id {
                        to_delete = false;
                        break;
                    }
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

    pub fn generate_map(&mut self, new_depth: usize) -> Point {
        // delete all entities
        let ids_to_delete = Self::entities_to_delete_on_level_change(&mut self.world);
        for id in ids_to_delete {
            self.world.delete_entity(id);
        }

        // Generate map
        let mut map_builder = match self.settings.mode {
            GameMode::VillageSim => crate::map_builders::village_builder(new_depth, self.settings.mapsize),
            GameMode::RL => crate::map_builders::rl_builder(new_depth, self.settings.mapsize),
            GameMode::OrcHalls => crate::map_builders::orc_halls_builder(new_depth, self.settings.mapsize),
            GameMode::MapDemo => crate::map_builders::random_builder(new_depth, self.settings.mapsize),
            GameMode::OrcArena => crate::map_builders::arena_builder(new_depth, self.settings.mapsize),
            GameMode::TestMode => crate::map_builders::village_builder(new_depth, self.settings.mapsize),
        };

        map_builder.build_map();

        let start_pos;
        {
            let mut map = self.world.borrow::<UniqueViewMut<Map>>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder.get_starting_position().ps.first().unwrap().clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(&mut self.world);

        // Update player position unique
        self.world.run(
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

        return start_pos;
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
        self.settings = settings;
        
        // Delete everything
        // world.clear();
        self.world = World::new();

        // Re-add defaults for all uniques
        self.world.add_unique(Map::new(settings.mapsize));
        self.world.add_unique(PPoint(Point::new(0, 0)));
        self.world.add_unique(Turn(0));
        self.world.add_unique(RNG(rltk::RandomNumberGenerator::new()));

        // make a player entity
        let player_id = self.world.run(|mut store: AllStoragesViewMut| 
            crate::entity_factory::player(&mut store, (0, 0), settings.show_player)
        );
        self.world.add_unique(PlayerID(player_id));

        self.world.add_unique(GameLog { messages: vec![] });
        self.world.add_unique(system_particle::ParticleBuilder::new());
        self.world.add_unique(FrameTime(0.));

        match settings.mode {
            GameMode::VillageSim => {
                // self.world.add_component(player_id, IsCamera {});
            }
            _ => {}
        }

        // Generate new map
        self.generate_map( 1);

        // give the player some items
        let e = self.world.run(|mut store: AllStoragesViewMut| {
            crate::entity_factory::magic_missile_scroll(&mut store, (0, 0))
        });
        add_effect(Some(player_id), EffectType::PickUp { entity: e });

        let e = self.world.run(|mut store: AllStoragesViewMut| {
            crate::entity_factory::dagger(&mut store, (0, 0))
        });
        add_effect(Some(player_id), EffectType::PickUp { entity: e });

        let e = self.world.run(|mut store: AllStoragesViewMut| {
            crate::entity_factory::fireball_scroll(&mut store, (0, 0))
        });
        add_effect(Some(player_id), EffectType::PickUp { entity: e });

        // run the gas system for a while to get the level nice and steamy
        for _ in 0..3000 {
            self.world.run(system_gas::run_gas_system);
        }

        self.run_systems();
    }
}
