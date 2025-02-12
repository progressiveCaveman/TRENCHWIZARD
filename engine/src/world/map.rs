use rltk::{Algorithm2D, Point, BaseMap, NavigationPath};
use serde::{Serialize, Deserialize};
use shipyard::{EntityId, View, Get, Unique, World};

use crate::{config::GameSettings, player, world::components::{Position, Renderable}, tiles::{GasType, TileRenderable, TileType, STABLE_GAS_AMOUNT}, ui::colors::{ColorUtils, COLOR_BG, COLOR_FIRE, COLOR_RED, COLOR_WHITE}, utils::Target, RenderOrder, DISABLE_FOV};

pub type XY = (i32, i32);
pub fn to_point(xy: XY) -> Point {
    Point::new(xy.0, xy.1)
}

#[derive(Default, Serialize, Deserialize, Clone, Unique)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub size: XY,
    pub blocked: Vec<bool>,
    pub fire_turns: Vec<i32>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<EntityId>>,
    pub vegetation: Vec<i32>,
    pub gases: Vec<(Vec<GasType>, usize)>, // usize - last neighbor that flowed into this space

    pub history: Vec<Vec<TileType>>,
}

impl Map {
    pub fn new(size: XY) -> Map {
        let count = (size.0 * size.1) as usize;
        Map {
            tiles: vec![TileType::Wall; count],
            size,
            blocked: vec![false; count],
            fire_turns: vec![0; count],
            tile_content: vec![Vec::new(); count],
            history: Vec::new(),
            vegetation: vec![0; count],
            gases: vec![(vec![GasType::Air; 7], 0); count],
        }
    }

    pub fn reset_tiles(&mut self, tile: TileType) {
        self.tiles.iter_mut().for_each(|t| *t = tile);
    }

    pub fn len(&self) -> usize {
        (self.size.0 * self.size.1) as usize
    }

    pub fn get_tile(&self, xy: XY) -> TileType {
        let idx = self.xy_idx(xy);
        self.tiles[idx]
    }

    pub fn set_tile(&mut self, xy: XY, value: TileType) {
        let idx = self.xy_idx(xy);
        self.tiles[idx] = value;
    }

    pub fn xy_idx(&self, xy: XY) -> usize {
        (xy.1 as usize * self.size.0 as usize) + xy.0 as usize
    }

    pub fn idx_xy(&self, idx: usize) -> XY {
        (idx as i32 % self.size.0, idx as i32 / self.size.0)
    }
    pub fn point_idx(&self, point: Point) -> usize {
        (point.y as usize * self.size.0 as usize) + point.x as usize
    }

    pub fn idx_point(&self, idx: usize) -> Point {
        Point::new(idx as i32 % self.size.0, idx as i32 / self.size.0)
    }

    pub fn in_bounds(&self, pos: XY) -> bool {
        pos.0 < self.size.0 && pos.1 < self.size.1 && pos.0 >= 0 && pos.1 >= 0
    }

    pub fn get_entity_renderable(&self, entities: &Vec<EntityId>, world: &World) -> Option<TileRenderable> {
        let vrend = world.borrow::<View<Renderable>>().unwrap();
        
        let mut render = None;//(' ', COLOR_BG, COLOR_BG);
        let mut current_order = RenderOrder::Items;

        for id in entities {
            if let Ok(rend) = vrend.get(*id) {
                if rend.order >= current_order {
                    render = Some((rend.glyph, rend.fg, rend.bg));
                    current_order = rend.order;
                }
            }
        }

        return render;
    }

    pub fn get_renderable(&self, pos: XY, settings: &GameSettings, world: &World, historyidx: Option<usize>) -> TileRenderable {
        let idx: usize = self.xy_idx(pos);

        if let Some(hidx) = historyidx {
            return self.history[usize::min(hidx, self.history.len() - 1)][idx].renderable();
        }

        let mut render = (' ', COLOR_BG, COLOR_BG);

        if settings.use_player_los && !DISABLE_FOV{
            if let Some(knowledge) = player::get_player_map_knowledge(world).get(&idx) {
                render = knowledge.0.renderable();

                if let Some(renderable) = self.get_entity_renderable(&knowledge.1, world) {
                    render = renderable;
                }
                
                render.1 = render.1.scale(0.5);
            }

            let vision = player::get_player_viewshed(world);
            if vision.visible_tiles.contains(&Point::new(pos.0, pos.1)) {
                render = self.tiles[idx].renderable();
                if self.fire_turns[idx] > 0 {
                    render = (render.0, render.1, COLOR_FIRE);
                }

                if let Some(renderable) = self.get_entity_renderable(&self.tile_content[idx], world) {
                    render = renderable;
                } 
            }
        } else {
            render = self.tiles[idx].renderable();
            
            if let Some(renderable) = self.get_entity_renderable(&self.tile_content[idx], world) {
                render = renderable;
            }
        }

        if self.fire_turns[idx] > 0 {
            render = (render.0, render.1, COLOR_FIRE);
        }

        let mut steam_count = 0.0;
        for gas in self.gases[idx].0.iter() {
            if *gas == GasType::Steam {
                steam_count += 1.0;
            }
        }

        if steam_count > 0.0 {
            render.2 = render.2.add(COLOR_WHITE.scale(f32::min(0.5 * steam_count / STABLE_GAS_AMOUNT as f32, 1.0)));
        } 
        
        // show areas with no air, maybe make this cause damage in future
        let xy = self.idx_xy(idx);
        if !self.is_wall(xy.0, xy.1) && self.gases[idx].0.len() < 3 {
            render.2 = render.2.add(COLOR_RED.scale(0.5));
        }

        return render;
    }

    // fn is_exit_valid(&self, x: usize, y: usize) -> bool {
    //     if x < 1 || x >= self.size.0 || y < 1 || y >= self.size.1 {
    //         return false;
    //     }
    //     return true;
    // }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx((x, y));
        self.tiles[idx] == TileType::Wall
            || self.tiles[idx] == TileType::WoodWall
            || self.tiles[idx] == TileType::WoodDoor
    }

    pub fn is_flammable(&self, idx: usize) -> bool {
        match self.tiles[idx] {
            TileType::Sand | TileType::Dirt | TileType::Stone | TileType::Floor | TileType::Grass | TileType::Wheat | TileType::WoodWall | 
            TileType::WoodDoor | TileType::WoodFloor => true,
            _ => false
        }
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

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x >= self.size.0 as i32 || y < 1 || y >= self.size.1 as i32{
            return false;
        }
        let idx = self.xy_idx((x, y));
        !self.blocked[idx]
    }

    pub fn get_path(&self, from: Point, to: Point) -> NavigationPath {
        // dbg!("Doing pathfinding, very slow");
        let path = rltk::a_star_search(self.point_idx(from) as i32, self.point_idx(to) as i32, self);

        return path;
    }

    pub fn gas_count(&self, idx: usize, gastype: GasType) -> usize {
        let mut steamcount = 0;
        self.gases[idx].0.iter().for_each(|gas| if *gas == gastype { steamcount += 1; });
        return steamcount;
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
        let (x, y) = (self.idx_xy(idx).0 as i32, self.idx_xy(idx).1 as i32);
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
