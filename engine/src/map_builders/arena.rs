use rltk::Point;
use shipyard::{AllStoragesViewMut, World};

use crate::{
    components::{Faction, SpawnerType},
    entity_factory, SHOW_MAPGEN_ANIMATION, tiles::TileType,
};

use super::{Map, MapBuilder, Position};

pub struct AernaBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
}

impl MapBuilder for AernaBuilder {
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
        world.run(|mut store: AllStoragesViewMut| {
            entity_factory::spawner(
                &mut store,
                4,
                self.map.size.1 / 2,
                Faction::Wizard1,
                SpawnerType::Orc,
                10,
            )
        });
        world.run(|mut store: AllStoragesViewMut| {
            entity_factory::spawner(
                &mut store,
                self.map.size.0 - 5,
                self.map.size.1 / 2,
                Faction::Wizard2,
                SpawnerType::Orc,
                10,
            )
        });
    }

    fn get_map_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            self.history.push(self.map.clone());
        }
    }
}

impl AernaBuilder {
    pub fn new(new_depth: usize, size: (usize, usize)) -> AernaBuilder {
        AernaBuilder {
            map: Map::new(size),
            starting_position: Position {
                ps: vec![Point::new(0, 0)],
            },
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        // let mut rng = RandomNumberGenerator::new();

        // set edges to be a wall
        for x in 0..self.map.size.0 {
            let idx = self.map.xy_idx((x, 0));
            self.map.tiles[idx] = TileType::Wall;

            let idx = self.map.xy_idx((x, self.map.size.1 - 1));
            self.map.tiles[idx] = TileType::Wall;
        }

        for y in 0..self.map.size.1 {
            let idx = self.map.xy_idx((0, y));
            self.map.tiles[idx] = TileType::Wall;

            let idx = self.map.xy_idx((self.map.size.0 - 1, y));
            self.map.tiles[idx] = TileType::Wall;
        }
        self.take_snapshot();

        // Set the map to grass with a river
        // for y in 1..self.map.size.1-1 {
        //     for x in 1..self.map.size.0-1 {
        //         let idx = self.map.xy_idx(x, y);

        //         if y > self.map.size.1 - 10 && y < self.map.size.1 - 3 {
        //             self.map.tiles[idx] = TileType::Water;
        //         } else {
        //             self.map.tiles[idx] = TileType::Grass;
        //         }
        //     }
        // }

        // First we completely randomize the map, setting 55% of it to be floor.
        // for y in 1..self.map.size.1/2 {
        //     for x in 1..self.map.size.0-1 {
        //         let roll = rng.roll_dice(1, 100);
        //         let idx = self.map.xy_idx(x, y);
        //         if roll > 55 { self.map.tiles[idx] = TileType::Floor }
        //         // else { self.map.tiles[idx] = TileType::Wall }
        //     }
        // }
        // self.take_snapshot();

        self.starting_position = Position {
            ps: vec![Point::new(self.map.size.0 / 2, self.map.size.1 / 2)],
        };

        return;
    }
}
