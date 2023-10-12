use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{entity_factory, SHOW_MAPGEN_ANIMATION, utils::rect::Rect, tiles::TileType};

use super::{common::apply_room_to_map, Map, MapBuilder, Position};

const MIN_ROOM_SIZE: i32 = 10;

const MIN_BUILDING_SIZE: i32 = 4;

pub struct BspFarmBuilder {
    map: Map,
    starting_position: Position,
    depth: usize,
    rooms: Vec<Rect>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspFarmBuilder {
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
            for room in self.rooms.iter().skip(1) {
                entity_factory::spawn_room(&mut store, &self.map, room, self.depth);
            }
        });
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            self.map.history.push(self.map.tiles.clone());
        }
    }
}

impl BspFarmBuilder {
    pub fn new(new_depth: usize, size: (usize, usize)) -> BspFarmBuilder {
        BspFarmBuilder {
            map: Map::new(size),
            starting_position: Position {
                ps: vec![Point::new(0, 0)],
            },
            depth: new_depth,
            rooms: Vec::new(),
            rects: Vec::new(),
        }
    }

    fn build(&mut self) {
        //convert interior tiles to dirt
        for x in 1..self.map.size.0 - 1 {
            for y in 1..self.map.size.1 - 1 {
                let idx = self.map.xy_idx((x, y));
                self.map.tiles[idx] = TileType::Dirt;
            }
        }

        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects
            .push(Rect::new(1, 1, self.map.size.0 as i32 - 2, self.map.size.1 as i32 - 2)); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects_recursive(first_room, &mut rng); // Divide the first room

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            self.rooms.push(room);

            // if room is on edge
            if room.x1 == 1 || room.y1 == 1 || room.x2 > self.map.size.0 as i32 - 5 || room.y2 > self.map.size.1 as i32 - 5 {
                let new_rect = Rect::new(room.x1 + 1, room.y1 + 1, room.width() - 2, room.height() - 2);
                apply_room_to_map(&mut self.map, &new_rect, TileType::Wheat, true);
            } else {
                if room.width() > MIN_BUILDING_SIZE + 1 && room.height() > MIN_BUILDING_SIZE + 1 {
                    let b_width = rng.range(MIN_BUILDING_SIZE, room.width());
                    let b_height = rng.range(MIN_BUILDING_SIZE, room.height());
                    let bx = rng.range(0, room.width() - b_width);
                    let by = rng.range(0, room.height() - b_height);
                    let room = Rect::new(room.x1 + bx, room.y1 + by, b_width, b_height);
                    apply_room_to_map(&mut self.map, &room, TileType::WoodWall, false);
                }
            }

            self.take_snapshot();
        }

        let start = self.rooms[0].center();
        self.starting_position = Position {
            ps: vec![Point { x: start.0, y: start.1 }],
        };

        // Don't forget the stairs
        let stairs = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx((stairs.0 as usize, stairs.1 as usize));
        self.map.tiles[stairs_idx] = TileType::StairsDown;
    }

    fn add_subrects_recursive(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calculate boundaries
        let width = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;

        let split: i32;
        if width < half_height {
            split = 3;
        } else if height < half_width {
            split = 2;
        } else {
            split = rng.roll_dice(1, 4);
        }

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new(rect.x1, rect.y1, half_width - 1, height);
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects_recursive(h1, rng);
            }
            let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, height);
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects_recursive(h2, rng);
            }
        } else {
            // Vertical split
            let v1 = Rect::new(rect.x1, rect.y1, width, half_height - 1);
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects_recursive(v1, rng);
            }
            let v2 = Rect::new(rect.x1, rect.y1 + half_height, width, half_height);
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects_recursive(v2, rng);
            }
        }
    }
}
