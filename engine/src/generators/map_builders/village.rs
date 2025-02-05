use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{
    simulator::components::{Faction, SpawnerType},
    entity_factory, SHOW_MAPGEN_ANIMATION, tiles::TileType, simulator::map::XY, ai::labors::{get_actions, AIBehaviors},
};

use super::{Map, MapBuilder, Position};

pub struct VillageBuilder {
    map: Map,
    starting_position: Position,
}

impl MapBuilder for VillageBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }
    fn build_map(&mut self) {
        self.build()
    }

    fn spawn_entities(&mut self, world: &mut World) {
        let mut rng = RandomNumberGenerator::new();
        world.run(|mut store: AllStoragesViewMut| {
            for y in 1..self.map.size.1 / 2 {
                for x in 1..self.map.size.0 - 1 {
                    let roll = rng.roll_dice(1, 100);
                    if roll < 35 {
                        entity_factory::tree(&mut store, (x, y));
                    }
                }
            }

            entity_factory::spawner(
                &mut store,
                (1, self.map.size.1 - 7),
                Faction::Nature,
                SpawnerType::Fish,
                1,
            );

            for i in 1..=10 {
                entity_factory::plank_house(&mut store, (20 + 10 * i, self.map.size.1 - 14), 4, 4);
            }

            entity_factory::chief_house(&mut store, (40, self.map.size.1 - 27), 20, 8);
            entity_factory::lumber_mill(&mut store, (20, self.map.size.1 - 27), 8, 8);
            entity_factory::fish_cleaner(&mut store, (10, self.map.size.1 - 17), 5, 5);

            for i in 0..20 {
                entity_factory::villager(&mut store, (15, self.map.size.1 - 25 - i), &get_actions(&vec![AIBehaviors::GatherWood, AIBehaviors::GatherFish]));
            }
        });
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            self.map.history.push(self.map.tiles.clone());
        }
    }
}

impl VillageBuilder {
    pub fn new(_new_depth: usize, size: XY) -> VillageBuilder {
        VillageBuilder {
            map: Map::new(size),
            starting_position: Position {
                ps: vec![Point::new(0, 0)],
            },
        }
    }

    fn build(&mut self) {
        // Set the map to grass with a river
        for y in 1..self.map.size.1 - 1 {
            for x in 1..self.map.size.0 - 1 {
                let idx = self.map.xy_idx((x, y));

                if y > self.map.size.1 - 10 && y < self.map.size.1 - 3 {
                    self.map.tiles[idx] = TileType::Water;
                } else {
                    self.map.tiles[idx] = TileType::Grass;
                }
            }
        }

        self.take_snapshot();

        self.starting_position = Position {
            ps: vec![Point::new(self.map.size.0 / 2, self.map.size.1 / 2)],
        };
    }
}
