use crate::world::map::XY;
use crate::tiles::{TileType, GasType};
use crate::utils::rect::Rect;
use crate::{entity_factory, SHOW_MAPGEN_ANIMATION};
use rltk::{Point, RandomNumberGenerator};
use shipyard::{AllStoragesViewMut, World};
use std::cmp;

use super::common::apply_drunkards_corrider;
use super::{Map, MapBuilder, Position};

pub struct DrunkardsBombingRunBuilder {
    map: Map,
    starting_position: Position,
    depth: usize,
    rooms: Vec<Rect>,
}

impl MapBuilder for DrunkardsBombingRunBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.rooms_and_corridors(50, 8, 12);
    }

    fn spawn_entities(&mut self, world: &mut World) {
        let mut rng = RandomNumberGenerator::new();

        world.run(|mut store: AllStoragesViewMut| {
            for room in self.rooms.iter().skip(1) {
                entity_factory::spawn_room(&mut store, &self.map, room, self.depth);
            }

            // pick 3 random spots each for gas adder and remover
            let mut i = 0;
            while i < 6 {
                let x: i32 = rng.range(1, self.map.size.0 as i32);
                let y: i32 = rng.range(1, self.map.size.1 as i32);
                let idx = self.map.xy_idx((x, y));

                if !self.map.is_wall(x, y) &&& self.map.tiles[idx] != &TileType::Water {
                    if i < 3 {
                        entity_factory::gas_adder(&mut store, (x, y), GasType::Steam);
                    } else {{
                        entity_factory::gas_remover(&mut store, (x, y));
                    }}
                    i += 1;
                }
            }
        });
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            self.map.history.push(self.map.tiles.clone());
        }
    }
}

impl DrunkardsBombingRunBuilder {
    pub fn new(new_depth: usize, size: XY) -> DrunkardsBombingRunBuilder {
        DrunkardsBombingRunBuilder {
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
                // apply_room_to_map(&mut self.map, &new_room, TileType::Floor, true);
                self.map.set_tile((new_room.center().0, new_room.center().1), TileType::Floor);

                self.rooms.push(new_room);
            }
            self.take_snapshot();
        }

        let mut room_candidates = self.rooms.clone();

        for i in 0..self.rooms.len() {
            let centerp = Point::new(self.rooms[i].center().0, self.rooms[i].center().1);
            let mut room: Rect = self.rooms[0];
            let mut dist = 1231231231231312.0;

            // find closest room
            if room_candidates.len() > 0 {
                room_candidates.remove(0);
                for r in room_candidates.iter() {
                    let rp = Point::new(r.center().0, r.center().1);

                    let newdist = rltk::DistanceAlg::Pythagoras.distance2d(centerp, rp);
                    if newdist < dist && newdist > 0. {
                        room = *r;
                        dist = newdist;
                    }
                }
            }

            let (x1, y1) = self.rooms[i].center();
            let (x2, y2) = room.center();

            apply_drunkards_corrider(&mut self.map, x1, y1, x2, y2);

            self.take_snapshot();
        }

        let stairs_down_pos = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx((stairs_down_pos.0, stairs_down_pos.1));
        self.map.tiles[stairs_idx] = TileType::StairsDown;

        let start_pos = self.rooms[0].center();
        self.starting_position = Position {
            ps: vec![Point {
                x: start_pos.0,
                y: start_pos.1,
            }],
        };
        self.take_snapshot();

        self.bomb_level();
        self.take_snapshot();

        // Find islands of walls and convert to other features
        let mut mapcpy = self.map.tiles.clone();

        //Remove border 'island'
        // let bi = self.get_flood_fill(&mapcpy, 0);
        // for i in 0..mapcpy.len() {
        //     if bi.0[i] {
        //         mapcpy[i] = TileType::Floor;
        //     }
        // }

        let mut islands: Vec<(Vec<bool>, i32)> = vec![];
        let mut first = true;
        for i in 0..mapcpy.len() {
            if mapcpy[i] == TileType::Wall {
                let island = self.get_flood_fill(&mapcpy, i);
                for i in 0..mapcpy.len() {
                    if island.0[i] {
                        mapcpy[i] = TileType::Floor;
                    }
                }

                if first {
                    first = false;
                } else {
                    islands.push(island);
                }
            }
        }

        // find largest island and convert to grass
        islands.sort_by(|a, b| b.1.cmp(&a.1));
        if !islands.is_empty() {
            let i0 = &islands.remove(0).0;
            for i in 0..self.map.tiles.len() {
                if i0[i] {
                    self.map.tiles[i] = TileType::Water;
                }
            }
        }

        // convert the rest of the islands
        for is in islands {
            for i in 0..self.map.tiles.len() {
                if is.0[i] {
                    self.map.tiles[i] = TileType::Grass;
                }
            }
        }
    }

    fn bomb_level(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let mut candidates: Vec<i32> = vec![];

        for i in 0..self.map.tiles.len() {
            let tile = self.map.tiles[i];
            let xy = self.map.idx_xy(i);

            if tile != TileType::Wall && xy.0 != 0 && xy.1 != 0 {
                candidates.push(i as i32);
            }
        }

        let iteration_number = candidates.len() as f32 * 3.8;

        for _ in 0..iteration_number as i32 {
            let mut random_offset: usize;

            // println!("cand len {}", candidates.len());

            // 1/3 chance that we will use as a bombing point one of the last 15 positions
            if rng.range(0, 3) == 0 {
                random_offset = rng.range(candidates.len() - cmp::min(2, 15), candidates.len() - 1);
            } else {
                // otherwise use lower half of remaining tiles
                random_offset = rng.range(0, candidates.len() / 2);
            }

            // check boundaries
            if random_offset >= candidates.len() {
                random_offset = candidates.len() - 1;
            }

            let idx = candidates[random_offset];
            let tx = self.map.idx_xy(idx as usize).0;
            let ty = self.map.idx_xy(idx as usize).1;
            let use_borders = true;

            // we will use bombs of radius 1 mostly with smaller chance (1/20)
            // that radius will be of size 2
            let bomb_radius = 1; //random_gen_get_i(20) != 0 ? 1 : 2;

            // bomb
            for x in cmp::max(0, tx - bomb_radius - 1)..cmp::min(self.map.size.0, tx + bomb_radius) {
                for y in cmp::max(0, ty - bomb_radius - 1)..cmp::min(self.map.size.1, ty + bomb_radius) {
                    // println!("bomb check {tx} {ty} {x} {y}");

                    // check if tile is within the circle
                    if (x - tx) * (x - tx) + (y - ty) * (y - ty) < bomb_radius * bomb_radius + bomb_radius {
                        if use_borders {
                            if x >= self.map.size.0 || y >= self.map.size.1 || x <= 0 || y <= 0 {
                                continue;
                            }
                        }

                        // if we have at least one tile bombed on screen
                        // push those coordinates to candidate list
                        let new_idx = self.map.xy_idx((x, y));
                        if self.map.tiles[new_idx] == TileType::Wall {
                            // self.map.set_tile(x, y, TileType::Floor);
                            self.map.tiles[new_idx] = TileType::Floor;
                            candidates.push(new_idx as i32);
                        }
                    }
                }
            }

            // erase our bombing cell, it is re-added in bombing loop above, if at least one tile is changed.
            candidates.drain(random_offset..random_offset + 1);
        }
    }

    // get all connected tiles of same type, and size of fill (number of trues in vec)
    pub fn get_flood_fill(
        &self,
        image: &Vec<TileType>,
        sidx: usize,
        // new_color: i32,
    ) -> (Vec<bool>, i32) {
        use std::collections::VecDeque;

        // let sr = usize::try_from(sr).unwrap();
        // let sc = usize::try_from(sc).unwrap();

        let mut ret = vec![false; image.len()];
        let mut count = 0;

        let initial_color = image[sidx];

        // if initial_color == new_color {
        //     return image;
        // }

        let mut cells: VecDeque<usize> = VecDeque::new();
        cells.push_back(sidx);

        while let Some(sidx) = cells.pop_front() {
            // let cell = image[sidx];

            if image[sidx] == initial_color && ret[sidx] == false {
                // *cell = new_color;
                ret[sidx] = true;
                count += 1;

                const OFFSETS: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];

                let (sr, sc) = self.map.idx_xy(sidx);

                for (delta_r, delta_c) in OFFSETS.iter().copied() {
                    let new_r = sr as i32 + delta_r;
                    let new_c = sc as i32 + delta_c;

                    if new_r < 0 || new_r >= self.map.size.0 as i32 || new_c < 0 || new_c >= self.map.size.1 as i32 {
                        continue;
                    }

                    let new_idx = self.map.xy_idx((new_r, new_c));

                    if image[new_idx] == initial_color {
                        cells.push_back(new_idx);
                    }
                }
            }
        }

        (ret, count)
    }
}
