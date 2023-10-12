use rltk::{Algorithm2D, Point, BaseMap};
use serde::{Serialize, Deserialize};
use shipyard::{Unique, EntityId, View, Get};

use crate::{components::Position, utils::Target, tiles::TileType};

#[derive(Default, Serialize, Deserialize, Clone, Unique)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub size: (usize, usize),
    pub blocked: Vec<bool>,
    pub fire_turns: Vec<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<EntityId>>,
}

impl Map {
    pub fn new(size: (usize, usize)) -> Map {
        let count = (size.0 * size.1) as usize;
        Map {
            tiles: vec![TileType::Water; count],
            size,
            blocked: Vec::new(),
            fire_turns: Vec::new(),
            tile_content: Vec::new(), 
        }
    }

    pub fn reset_tiles(&mut self, tile: TileType) {
        self.tiles.iter().for_each(|mut t| t = &tile);
    }

    pub fn len(&self) -> usize {
        self.size.0 * self.size.1
    }

    pub fn get_tile(&self, xy: (usize, usize)) -> TileType {
        let idx = self.xy_idx(xy);
        self.tiles[idx]
    }

    pub fn set_tile(&mut self, xy: (usize, usize), value: TileType) {
        let idx = self.xy_idx(xy);
        self.tiles[idx] = value;
    }

    pub fn xy_idx(&self, xy: (usize, usize)) -> usize {
        (xy.1 as usize * self.size.0 as usize) + xy.0 as usize
    }

    pub fn idx_xy(&self, idx: usize) -> (usize, usize) {
        (idx as usize % self.size.0, idx as usize / self.size.0)
    }
    pub fn point_idx(&self, point: Point) -> usize {
        (point.y as usize * self.size.0 as usize) + point.x as usize
    }

    pub fn idx_point(&self, idx: usize) -> Point {
        Point::new(idx % self.size.0, idx / self.size.0)
    }

    pub fn in_bounds(&self, pos: (usize, usize)) -> bool {
        pos.0 < self.size.0 && pos.1 < self.size.1
    }

    // fn is_exit_valid(&self, x: usize, y: usize) -> bool {
    //     if x < 1 || x >= self.size.0 || y < 1 || y >= self.size.1 {
    //         return false;
    //     }
    //     return true;
    // }

    pub fn get_glyph(&self, p: (usize, usize)) -> char {
        match self.tiles[self.xy_idx(p)] {
            TileType::Water => '~',
            TileType::Sand => '.',
            TileType::Dirt => '.',
            TileType::Stone => '#',
            _ => '!'
        }
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        let idx = self.xy_idx((x, y));
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

    pub fn distance(&self, vpos: &View<Position>, f: Target, t: Target) -> f32 {
        let idx1 = match f {
            Target::LOCATION(l) => vec![self.point_idx(Point::new(l.x, l.y))],
            Target::ENTITY(e) => {
                if let Ok(p) = vpos.get(e) {
                    p.idxes(self)
                } else {
                    vec![0]
                }
            }
        };

        let idx2 = match t {
            Target::LOCATION(l) => vec![self.point_idx(Point::new(l.x, l.y))],
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

    fn is_exit_valid(&self, x: usize, y: usize) -> bool {
        if x < 1 || x >= self.size.0 || y < 1 || y >= self.size.1 {
            return false;
        }
        let idx = self.xy_idx((x, y));
        !self.blocked[idx]
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.size.0, self.size.1)
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
        let w = self.size.0 as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let (x, y) = self.idx_xy(idx);
        let w = self.size.0 as usize;

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
