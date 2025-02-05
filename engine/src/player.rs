use std::collections::HashMap;

use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, ViewMut, World};

use crate::simulator::components::{SpatialKnowledge, Vision, PlayerID, PPoint, GameLog};
use crate::map::Map;
use crate::tiles::TileType;
use crate::utils::InvalidPoint;

pub fn get_player_map_knowledge(world: &World) -> HashMap<usize, (TileType, Vec<EntityId>)> {
    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    if let Ok(vspace) = world.borrow::<ViewMut<SpatialKnowledge>>() {
        if let Ok(space) = vspace.get(player_id) {
            return space.tiles.clone();
        }
    }

    HashMap::new()
}

pub fn get_player_viewshed(world: &World) -> Vision {
    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    let vvs = world.borrow::<ViewMut<Vision>>().unwrap();

    if let Ok(vs) = vvs.get(player_id) {
        vs.clone()
    } else {
        Vision {
            visible_tiles: Vec::new(),
            range: 0,
            dirty: true,
        }
    }
}

pub fn reveal_map(world: &World) {
    // let world = &gs.world;
    // let res = &gs.resources;
    let map = world.borrow::<UniqueView<Map>>().unwrap();
    let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    if let Ok(mut vspace) = world.borrow::<ViewMut<SpatialKnowledge>>() {
        if let Ok(space) = (&mut vspace).get(player_id) {
            for i in 0..map.tiles.len() {
                space.tiles.insert(i, (map.tiles[i], map.tile_content[i].clone()));
            }
        }
    }
}

pub fn try_next_level(world: &World) -> bool {
    let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
    let map = world.borrow::<UniqueView<Map>>().unwrap();
    let player_idx = map.xy_idx(player_pos.to_xy());
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    } else {
        let mut log = world.borrow::<UniqueViewMut<GameLog>>().unwrap();
        log.messages.push(format!("There is no stairs down here"));
        false
    }
}
