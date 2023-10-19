use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{entity_factory, SHOW_MAPGEN_ANIMATION, utils::rect::Rect, tiles::TileType, map::XY};

use super::{Map, MapBuilder, Position};

const MIN_ROOM_SIZE: i32 = 4;

pub struct BspInteriorBuilder {
    map: Map,
    starting_position: Position,
    depth: usize,
    rooms: Vec<Rect>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
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

impl BspInteriorBuilder {
    pub fn new(new_depth: usize, size: XY) -> BspInteriorBuilder {
        BspInteriorBuilder {
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
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects
            .push(Rect::new(1, 1, self.map.size.0 as i32 - 2, self.map.size.1 as i32 - 2)); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, &mut rng); // Divide the first room

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            //room.x2 -= 1;
            //room.y2 -= 1;
            self.rooms.push(room);
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = self.map.xy_idx((x, y));
                    if idx > 0 && idx < ((self.map.size.0 * self.map.size.1) - 1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.take_snapshot();
        }

        let start = self.rooms[0].center();
        self.starting_position = Position {
            ps: vec![Point { x: start.0, y: start.1 }],
        };

        // Now we want corridors
        for i in 0..self.rooms.len() - 1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        // Don't forget the stairs
        let stairs = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx((stairs.0, stairs.1));
        self.map.tiles[stairs_idx] = TileType::StairsDown;
    }

    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calculate boundaries
        let width = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new(rect.x1, rect.y1, half_width - 1, height);
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            }
            let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, height);
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            }
        } else {
            // Vertical split
            let v1 = Rect::new(rect.x1, rect.y1, width, half_height - 1);
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            }
            let v2 = Rect::new(rect.x1, rect.y1 + half_height, width, half_height);
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            }
        }
    }

    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }

            let idx = self.map.xy_idx((x, y));
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}
