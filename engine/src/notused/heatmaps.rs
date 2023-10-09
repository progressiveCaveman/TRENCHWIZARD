use crate::{map::{Map, TileType}, State, components::{Item, ItemType, Tree, LumberMill, FishCleaner}};

const MAX_DEPTH: f32 = 400.0;

pub enum HeatMapType {
    Water,
    Logs,
    Trees,
    LumberMills,
    Fisheries,
}

#[derive(Default, Clone)]
pub struct HeatMaps {
    pub water: Vec<f32>,
    pub logs: Vec<f32>,
    pub trees: Vec<f32>,
    pub lumbermills: Vec<f32>,
    pub fisheries: Vec<f32>,
}

impl HeatMaps {
    pub fn new() -> HeatMaps {
        HeatMaps {
            water: Vec::new(),
            logs: Vec::new(),
            trees: Vec::new(),
            lumbermills: Vec::new(),
            fisheries: Vec::new(),
        }
    }

    pub fn refresh_heat_maps(&mut self, gs: &mut State) {
        let world = &gs.world;
        let map: &Map = &gs.resources.get_mut::<Map>().unwrap();

        let mapsize = map.tiles.len();

        let mut waterstarts: Vec<usize> = vec![];
        let mut treestarts: Vec<usize> = vec![];
        let mut logstarts: Vec<usize> = vec![];
        let mut lumbermillstarts: Vec<usize> = vec![];
        let mut fisheriesstarts: Vec<usize> = vec![];

        for idx in 0..mapsize {
            if map.tiles[idx] == TileType::Water { waterstarts.push(idx); }
            for entity in map.tile_content[idx].iter() {
                if let Ok(item) = world.get::<Item>(*entity) {
                    if item.typ == ItemType::Log {
                        logstarts.push(idx);
                    }
                }

                if let Ok(_) = world.get::<Tree>(*entity) {
                    treestarts.push(idx);
                }

                if let Ok(_) = world.get::<LumberMill>(*entity) {
                    lumbermillstarts.push(idx);
                }

                if let Ok(_) = world.get::<FishCleaner>(*entity) {
                    fisheriesstarts.push(idx);
                }
            }
        }

        self.water = rltk::DijkstraMap::new(map.width, map.height, &waterstarts, map, MAX_DEPTH).map;
        self.logs = rltk::DijkstraMap::new(map.width, map.height, &logstarts, map, MAX_DEPTH).map;
        self.trees = rltk::DijkstraMap::new(map.width, map.height, &treestarts, map, MAX_DEPTH).map;
        self.lumbermills = rltk::DijkstraMap::new(map.width, map.height, &lumbermillstarts, map, MAX_DEPTH).map;
        self.fisheries = rltk::DijkstraMap::new(map.width, map.height, &fisheriesstarts, map, MAX_DEPTH).map;
    }
}