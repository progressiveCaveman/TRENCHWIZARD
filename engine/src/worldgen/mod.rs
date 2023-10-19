
use rand::Rng;

use crate::{map::{Map, XY}, tiles::TileType};

pub fn basic_fill(map: &mut Map) {
    let size = map.size;

    for i in 0..map.tiles.len() {
        map.tiles[i] = TileType::Water;
    }

    for i in 0..240 {
        let start1 = map.xy_idx(rnd_point(size));
        if i % 3 == 0 {
            map.tiles[start1] = TileType::Dirt;
        } else if i % 3 == 1 {
            map.tiles[start1] = TileType::Sand;
        } else {
            map.tiles[start1] = TileType::Stone;
        }
    }

    fill_recursive(map, 0);
}

fn fill_recursive(map: &mut Map, depth: usize) {
    if depth > 200 {
        return;
    }

    let mut new: Vec<TileType> = vec![TileType::Water; map.tiles.len()];
    let mut water = 0;

    for index in 0..map.tiles.len() {
        let tile = map.tiles[index];
        new[index] = tile;

        if tile == TileType::Water {
            water += 1;

            let neighbors = get_neighbors(map.idx_xy(index));

            for p in neighbors.iter() {
                if map.in_bounds((p.0, p.1)) {
                    let idx = map.xy_idx(*p);
                    let t = map.tiles[idx];
                    if t != TileType::Water {
                        new[index] = t;
                        break;
                    }
                }
            }
        }
    }

    map.tiles = new;

    if water > 100 {
        fill_recursive(map, depth + 1);
    }
}

fn rnd_point(size: XY) -> XY {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    let y: f32 = rng.gen();
    ((x * size.0 as f32) as i32, (y * size.1 as f32) as i32)
}

pub fn get_neighbors(point: XY) -> Vec<XY> {
    let mut n = vec![];

    if point.0 > 0 {
        n.push((point.0 - 1, point.1));
        n.push((point.0 - 1, point.1 + 1));
    }

    if point.0 > 0 && point.1 > 0 {
        n.push((point.0 - 1, point.1 - 1));
    }

    if point.1 > 0 {
        n.push((point.0, point.1 - 1));
        n.push((point.0 + 1, point.1 - 1));

    }

    n.push((point.0,point.1 + 1));
    n.push((point.0 + 1,point.1));
    n.push((point.0 + 1,point.1 + 1));

    n
}
