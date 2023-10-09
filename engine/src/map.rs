use rltk::{self};
use rltk::{Algorithm2D, BaseMap, Point};
use serde;
use serde::{Deserialize, Serialize};
use shipyard::{EntityId, Get, Unique, View};

use crate::ai::decisions::Target;
use crate::components::Position;
use crate::{OFFSET_X, OFFSET_Y, SCALE};

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    StairsDown,
    StairsUp,
    Grass,
    Wheat,
    Dirt,
    Sand,
    Stone,
    Water,
    WoodWall,
    WoodDoor,
    WoodFloor,
}

#[derive(Default, Serialize, Deserialize, Clone, Unique)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub blocked: Vec<bool>,
    pub fire_turns: Vec<i32>,
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<EntityId>>,

    // vec of numbers for debug. If it's not set, shouldn't affect anything
    pub dijkstra_map: Vec<f32>,
    // TODO Maybe this doesn't belong here, a system would be better practice (but uglier)
    // pub influence_maps: Vec<Vec<f32>>,
}

impl Map {
    pub fn new(new_depth: i32, tile_type: TileType, size: (i32, i32)) -> Map {
        let count = (size.0 * size.1) as usize;
        Map {
            tiles: vec![tile_type; count],
            width: size.0,
            height: size.1,
            blocked: vec![false; count],
            fire_turns: vec![0; count],
            tile_content: vec![Vec::new(); count],
            depth: new_depth,
            dijkstra_map: vec![-1.0; count],
            // influence_maps:vec![vec![0.0; count]; 2],// todo magic numbers
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, value: TileType) {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] = value;
    }

    pub fn get_tile(&self, pos: (usize, usize)) -> TileType {
        self.tiles[self.xy_idx(pos.0 as i32, pos.1 as i32)]
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn point_idx(&self, point: Point) -> usize {
        (point.y as usize * self.width as usize) + point.x as usize
    }

    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
    }

    pub fn idx_point(&self, idx: usize) -> Point {
        Point {
            x: idx as i32 % self.width,
            y: idx as i32 / self.width,
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn in_bounds_new(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.width as usize && pos.1 < self.height as usize
    }

    pub fn get_glyph(&self, p: (usize, usize)) -> char {
        match self.tiles[self.xy_idx(p.0 as i32, p.1 as i32)] {
            TileType::Water => '~',
            TileType::Sand => '.',
            TileType::Dirt => '.',
            TileType::Stone => '#',
            _ => unimplemented!()
        }
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] == TileType::Wall
            || self.tiles[idx] == TileType::WoodWall
            || self.tiles[idx] == TileType::WoodDoor
    }

    pub fn is_flammable(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Grass
            || self.tiles[idx] == TileType::Wheat
            || self.tiles[idx] == TileType::WoodWall
            || self.tiles[idx] == TileType::WoodDoor
    }

    pub fn blocks_movement(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
            || self.tiles[idx] == TileType::Water
            || self.tiles[idx] == TileType::WoodWall
            || self.tiles[idx] == TileType::WoodDoor
    }

    pub fn set_blocked(&mut self) {
        for (i, _t) in self.tiles.iter().enumerate() {
            self.blocked[i] = self.blocks_movement(i);
        }
    }

    pub fn clear_tile_content(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    // this only works if ctx.set_active_console is set correctly
    pub fn transform_mouse_pos(&self, mouse_pos: (i32, i32)) -> (i32, i32) {
        (
            mouse_pos.0 - (OFFSET_X as f32 / SCALE).ceil() as i32,
            mouse_pos.1 - (OFFSET_Y as f32 / SCALE).ceil() as i32,
        )
    }

    pub fn mouse_in_bounds(&self, mouse_pos: (i32, i32)) -> bool {
        mouse_pos.0 >= 0 && mouse_pos.0 <= self.width && mouse_pos.1 >= 0 && mouse_pos.1 <= self.height
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x >= self.width || y < 1 || y >= self.height {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn distance(&self, vpos: &View<Position>, f: Target, t: Target) -> f32 {
        let idx1 = match f {
            Target::LOCATION(l) => vec![self.xy_idx(l.x, l.y)],
            Target::ENTITY(e) => {
                if let Ok(p) = vpos.get(e) {
                    p.idxes(self)
                } else {
                    vec![0]
                }
            }
        };

        let idx2 = match t {
            Target::LOCATION(l) => vec![self.xy_idx(l.x, l.y)],
            Target::ENTITY(e) => {
                if let Ok(p) = vpos.get(e) {
                    p.idxes(self)
                } else {
                    vec![0]
                }
            }
        };

        let mut min = f32::MAX;
        for i1 in idx1.iter() {
            for i2 in idx2.iter() {
                let dist = self.get_pathing_distance(*i1, *i2);
                if dist < min {
                    min = dist;
                }
            }
        }

        min
    }

    // pub fn refresh_water_map(&mut self) {
    //     let mut waters: Vec<usize> = vec![];
    //     for i in 0..self.tiles.len() {
    //         if self.tiles[i] == TileType::Water {
    //             waters.push(i);
    //         }
    //     }

    //     self.water_map = rltk::DijkstraMap::new(self.width, self.height, &waters, self, 800.0).map;
    // }

    // pub fn refresh_influence_maps(&mut self, gs: &State, turn: i32){
    //     if turn % 10 == 0 {
    //         let unit_str = 100;

    //         let mut f1: Vec<(Point, f32)> = Vec::new();
    //         let mut f2: Vec<(Point, f32)> = Vec::new();

    //         for (_, (pos, faction)) in gs.world.query::<(&Position, &Faction)>().iter() {
    //             for p in pos.ps.iter() {
    //                 match faction.faction {
    //                     1 => f1.push((*p, unit_str as f32)),
    //                     2 => f2.push((*p, unit_str as f32)),
    //                     _ => {}
    //                 }
    //             }
    //         }

    //         self.repopulate_influence_map(f1, 0.9, 0);
    //         self.repopulate_influence_map(f2, 0.9, 1);
    //     }
    // }

    // pub fn repopulate_influence_map(&mut self, pois: Vec<(Point, f32)>, spread: f32, imap_index: usize) {
    //     for i in 0..self.influence_maps[imap_index].len() {
    //         self.influence_maps[imap_index][i] = 0.0;
    //     }

    //     // let vals = &mut self.influence_maps[Inf_map_index];

    //     // add poi vals to map
    //     for poi in pois.iter() {
    //         let idx = self.xy_idx(poi.0.x, poi.0.y);
    //         self.influence_maps[imap_index][idx] = poi.1;
    //     }

    //     // return;

    //     // iterate on the map and blur all influence
    //     let num_iterations = 10;
    //     for _ in 0..num_iterations {
    //         // buffer to hold the changes for this step
    //         let mut buf = vec![0.0; self.influence_maps[imap_index].len()];

    //         let mut max_inf: f32 = 0.0;

    //         // reduce each tile
    //         for (i, el) in self.influence_maps[imap_index].iter().enumerate() {
    //             max_inf = f32::max(max_inf, *el);
    //             let neighbors = self.get_available_exits(i);

    //             // get amount to decay by
    //             let decay_amount:f32 = self.influence_maps[imap_index][i] * spread;
    //             buf[i] = buf[i] - decay_amount;

    //             // distribute decay amongst the neighborhood
    //             for n in neighbors.iter() {
    //                 if n.0 >= buf.len() {
    //                     continue;
    //                 }

    //                 //  float inf = m_Influences[c.neighbor] * expf(-c.dist * m_fDecay);
    //                 buf[n.0] = buf[n.0] + decay_amount / neighbors.len() as f32;
    //             }
    //         }

    //         // apply change buffer to values
    //         for i in 0..buf.len() {
    //             self.influence_maps[imap_index][i] += buf[i];
    //         }
    //     }
    // }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
            || self.tiles[idx] == TileType::Wheat
            || self.tiles[idx] == TileType::WoodWall
            || self.tiles[idx] == TileType::WoodDoor // TODO make fire block too?
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let (x, y) = self.idx_xy(idx);
        let w = self.width as usize;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - w + 1, 1.45))
        };
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + w + 1, 1.45))
        };

        exits
    }
}
