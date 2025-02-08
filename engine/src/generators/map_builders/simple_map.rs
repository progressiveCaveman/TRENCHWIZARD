use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};

use crate::{entity_factory, SHOW_MAPGEN_ANIMATION, utils::rect::Rect, tiles::TileType, world::map::XY};

use super::{
    apply_horizontal_corridor, apply_room_to_map, apply_vertical_corridor, Map, MapBuilder, Position,
};

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    depth: usize,
    rooms: Vec<Rect>,
}

impl MapBuilder for SimpleMapBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors(10, 4, 8);
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

impl SimpleMapBuilder {
    pub fn new(new_depth: usize, size: XY) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(size),
            starting_position: Position {
                ps: vec![Point::new(0, 0)],
            },
            depth: new_depth,
            rooms: Vec::new(),
        }
    }

    fn rooms_and_corridors(&mut self, max_rooms: i32, min_size: i32, max_size: i32) {
        let mut rng = RandomNumberGenerator::new();

        self.take_snapshot();
        for _ in 0..max_rooms {
            let w: i32 = rng.range(min_size, max_size);
            let h: i32 = rng.range(min_size, max_size);
            let x: i32 = rng.range(1, self.map.size.0 as i32 - w - 1);
            let y: i32 = rng.range(1, self.map.size.1 as i32 - h - 1);

            let new_room = Rect::new(x, y, w, h);
            let mut place_room = true;

            for other_room in self.rooms.iter() {
                if new_room.intersect(&other_room) {
                    place_room = false;
                }
            }

            if place_room {
                apply_room_to_map(&mut self.map, &new_room, TileType::Floor, true);
                self.rooms.push(new_room);
            }
            self.take_snapshot();
        }

        for i in 1..self.rooms.len() {
            let (x1, y1) = self.rooms[i].center();
            let (x2, y2) = self.rooms[i - 1].center();

            apply_horizontal_corridor(&mut self.map, x1, x2, y1);
            apply_vertical_corridor(&mut self.map, x2, y1, y2);
            apply_vertical_corridor(&mut self.map, x1, y1, y2);
            apply_horizontal_corridor(&mut self.map, x1, x2, y2);

            self.take_snapshot();
        }

        let stairs_down_pos = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx((stairs_down_pos.0, stairs_down_pos.1));
        self.map.tiles[stairs_idx] = TileType::StairsDown;

        // remove_useless_walls(&mut self.map);

        let start_pos = self.rooms[0].center();
        self.starting_position = Position {
            ps: vec![Point {
                x: start_pos.0,
                y: start_pos.1,
            }],
        };
        self.take_snapshot();
    }
}
