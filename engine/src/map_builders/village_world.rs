use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{entity_factory, SHOW_MAPGEN_ANIMATION};

use super::{Map, MapBuilder, Position, TileType};

pub struct VillageWorldBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
}

impl MapBuilder for VillageWorldBuilder {
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

        let mut used_idx = vec![];

        for _ in 0..100 {
            let x = rng.roll_dice(1, self.map.width - 1);
            let y = rng.roll_dice(1, self.map.height - 1);
            let idx = self.map.xy_idx(x, y);
            if !self.map.is_wall(x, y) && self.map.tiles[idx] != TileType::Water && !used_idx.contains(&idx) {
                used_idx.push(idx);
                world.run(|mut store: AllStoragesViewMut| {
                    entity_factory::villager(&mut store, x, y);
                });
            }
        }

        // world.run(|mut store: AllStoragesViewMut|{

        //     for y in 1..self.map.height/2 {
        //         for x in 1..self.map.width-1 {
        //             let roll = rng.roll_dice(1, 100);
        //             if roll < 35 {
        //                 entity_factory::tree(&mut store, x, y);
        //             }
        //         }
        //     }

        //     entity_factory::spawner(&mut store, 1, self.map.height - 7, 0, SpawnerType::Fish, 1);

        //     for i in 1..=10 {
        //         entity_factory::plank_house(&mut store, 20 + 10 * i, self.map.height - 14, 4, 4);
        //     }

        //     entity_factory::chief_house(&mut store, 40, self.map.height - 27, 20, 8);
        //     entity_factory::lumber_mill(&mut store, 20, self.map.height - 27, 8, 8);
        //     entity_factory::fish_cleaner(&mut store, 10, self.map.height - 17, 5, 5);

        //     for i in 0..20{
        //         entity_factory::villager(&mut store, 15, self.map.height - 25 - i);
        //     }
        // });
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

impl VillageWorldBuilder {
    pub fn new(new_depth: i32, size: (i32, i32)) -> VillageWorldBuilder {
        VillageWorldBuilder {
            map: Map::new(new_depth, TileType::Wall, size),
            starting_position: Position {
                ps: vec![Point { x: 0, y: 0 }],
            },
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        // Set the map to grass
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.xy_idx(x, y);
                self.map.tiles[idx] = TileType::Grass;
            }
        }

        self.take_snapshot();

        let villsize = (150, 80);

        // let numvillhori = self.map.width / villsize.0;
        // let numvillvert = self.map.height / villsize.1;

        let mut x = 0;
        let mut y = 0;
        while x <= self.map.width - villsize.0 {
            // todo test with width multiple of villwidth
            while y <= self.map.height - villsize.1 {
                let mut villbuilder = super::village_builder(0, villsize);
                villbuilder.build_map();

                let map = villbuilder.get_map();

                for i in 0..map.tiles.len() {
                    let pos = map.idx_point(i);
                    let targetpos = Point {
                        x: x + pos.x,
                        y: y + pos.y,
                    };
                    let targetposidx = self.map.point_idx(targetpos);
                    self.map.tiles[targetposidx] = map.tiles[i];
                }

                y += villsize.1;

                self.take_snapshot();
            }

            x += villsize.0;
        }

        // // Set the map to grass with a river
        // for y in 1..self.map.height-1 {
        //     for x in 1..self.map.width-1 {
        //         let idx = self.map.xy_idx(x, y);

        //         if y > self.map.height - 10 && y < self.map.height - 3 {
        //             self.map.tiles[idx] = TileType::Water;
        //         } else {
        //             self.map.tiles[idx] = TileType::Grass;
        //         }
        //     }
        // }

        self.take_snapshot();

        self.starting_position = Position {
            ps: vec![Point {
                x: self.map.width / 2,
                y: self.map.height / 2,
            }],
        };
    }
}
