use crate::{utils::rect::Rect, tiles::TileType};

use super::Map;
use rltk::RandomNumberGenerator;
use std::cmp;

pub fn rect_in_bounds(map: &mut Map, room: &Rect) -> bool {
    map.in_bounds((room.x1 as usize, room.y1 as usize)) && map.in_bounds((room.x2 as usize, room.y2 as usize))
}

pub fn apply_room_to_map(map: &mut Map, room: &Rect, tile_type: TileType, filled: bool) {
    if !rect_in_bounds(map, room) {
        return;
    }

    for y in room.y1..room.y2 {
        for x in room.x1..room.x2 {
            if filled || x == room.x1 || x == room.x2 - 1 || y == room.y1 || y == room.y2 - 1 {
                map.set_tile((x as usize, y as usize), tile_type);
            }
        }
    }
}

pub fn apply_vertical_corridor(map: &mut Map, x: i32, y1: i32, y2: i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map.set_tile((x as usize, y as usize), TileType::Floor);
    }
}

pub fn apply_horizontal_corridor(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map.set_tile((x as usize, y as usize), TileType::Floor);
    }
}

pub fn apply_drunkards_corrider(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut rng = RandomNumberGenerator::new();

    let mut xdir = 1;
    let mut ydir = 1;

    if x1 > x2 {
        xdir = -1;
    }

    if y1 > y2 {
        ydir = -1;
    }

    let mut x = x1;
    let mut y = y1;

    while x != x2 || y != y2 {
        if x == x2 {
            y += ydir;
        } else if y == y2 {
            x += xdir;
        } else {
            if rng.range(0, 2) == 0 {
                x += xdir;
            } else {
                y += ydir;
            }
        }
        map.set_tile((x as usize, y as usize), TileType::Floor);
    }
}

// pub fn remove_useless_walls(map: &mut Map) {
//     let mut to_remove: Vec<(i32, i32)> = Vec::new();

//     for i in 0..map.tiles.len() {
//         let (x, y) = map.idx_xy(i);

//         if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 { continue }

//         if map.is_wall(x - 1, y - 1) && map.is_wall(x, y - 1) && map.is_wall(x + 1, y - 1) &&
//            map.is_wall(x - 1, y    ) && map.is_wall(x, y    ) && map.is_wall(x + 1, y    ) &&
//            map.is_wall(x - 1, y + 1) && map.is_wall(x, y + 1) && map.is_wall(x + 1, y + 1) {
//             to_remove.push((x, y));
//         }
//     }

//     for (x, y) in to_remove {
//         map.set_tile(x, y, TileType::Floor);
//     }
// }
