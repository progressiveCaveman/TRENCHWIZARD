use std::collections::HashMap;

use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{entity_factory, SHOW_MAPGEN_ANIMATION};

use super::{Map, MapBuilder, Position, TileType};

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
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
            for area in self.noise_areas.iter() {
                entity_factory::spawn_region(&mut store, area.1, self.depth);
            }
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

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32, size: (i32, i32)) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth, TileType::Wall, size),
            starting_position: Position {
                ps: vec![Point { x: 0, y: 0 }],
            },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                if roll > 55 {
                    self.map.tiles[idx] = TileType::Floor
                } else {
                    self.map.tiles[idx] = TileType::Wall
                }
            }
        }
        self.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut p = Point {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self.map.xy_idx(p.x, p.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            p.x -= 1;
            start_idx = self.map.xy_idx(p.x, p.y);
        }
        self.starting_position = Position { ps: vec![p] };

        // Find all tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height, &map_starts, &self.map, 200.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::StairsDown;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.xy_idx(x, y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }
    }
}
