use crate::components::{Position, AddsGas, RNG, RemovesGas};
use crate::map::Map;
use crate::tiles::{GasType, MAX_GAS};
use crate::utils::{InvalidPoint, get_neighbors};
use shipyard::{IntoIter, IntoWithId, View, ViewMut, UniqueViewMut};

pub fn run_gas_system(
    mut map: UniqueViewMut<Map>,
    mut rng: UniqueViewMut<RNG>,
    vpos: View<Position>,
    mut vaddsgas: ViewMut<AddsGas>,
    mut vremovesgas: ViewMut<RemovesGas>,
) {
    // init map gases
    for idx in 0..map.tiles.len() {
        if map.gases[idx].1 == 0 {
            map.gases[idx].1 = idx;

            let xy = map.idx_xy(idx);
            if map.is_wall(xy.0, xy.1) {
                map.gases[idx].0 = vec![GasType::Blocked; MAX_GAS];
            }
        }
    }

    let mut new_gases = map.gases.clone();// todo clone slow

    // run gas dissipation
    for (idx, _) in map.gases.iter().enumerate() {
        let point = map.idx_point(idx);
        let steamcount = map.gas_count(idx, GasType::Steam);

        if steamcount > 0 {
            let mut flow_target = 0;
            let mut flow_best = 0.0;
            for &n in get_neighbors(point).iter().filter(|p| map.in_bounds(p.to_xy())) {
                let nidx = map.point_idx(n);

                let dist = 1.0;//map.get_pathing_distance(*lastflow, nidx);
                let air_amount = map.gas_count(nidx, GasType::Air) as f32 / MAX_GAS as f32;
                let none_amount = map.gas_count(nidx, GasType::None);

                let mut score = air_amount * dist * rng.0.roll_dice(1, 5) as f32 / 10.0;
                
                if none_amount > 0 {
                    score *= 10.0;
                }

                for gas in map.gases[nidx].0.iter() {
                    if *gas == GasType::Air {
                        if score > flow_best {
                            flow_target = nidx;
                            flow_best = score;
                        }
                        break;
                    }
                }
            }
    
            if flow_target != 0 && rng.0.roll_dice(1, steamcount as i32) as f32 / MAX_GAS as f32 > 0.3 {
                for gas in new_gases[idx].0.iter_mut() {
                    if *gas == GasType::Steam {
                        *gas = GasType::Air;
                        break;
                    }
                }

                new_gases[flow_target].1 = idx; // set flow
                for gas in new_gases[flow_target].0.iter_mut() {
                    if *gas == GasType::Air {
                        *gas = GasType::Steam;
                        break;
                    }
                }
            }
        }
    }

    // Run AddsGas components
    for (_, (pos, addsgas)) in (&vpos, &mut vaddsgas).iter().with_id() {
        let idx = map.point_idx(pos.ps[0]);
        for gas in new_gases[idx].0.iter_mut() {
            if *gas == GasType::Air {
                *gas = addsgas.gas;
                break;
            }
        }
    }

    // Run removes gas components
    for (_, (pos, _)) in (&vpos, &mut vremovesgas).iter().with_id() {
        let idx = map.point_idx(pos.ps[0]);
        for gas in new_gases[idx].0.iter_mut() {
            if *gas != GasType::Air {
                *gas = GasType::Air;
                break;
            }
        }
    }

    //replace old gases
    map.gases = new_gases;

    // for (id, (pos, vs)) in (&vpos, &mut vvs).iter().with_id() {
    //     // if vs.dirty {
    //     let pos = pos.ps.first().unwrap();

    //     vs.dirty = false;
    //     vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
    //     vs.visible_tiles
    //         .retain(|p| p.x >= 0 && p.x < map.size.0 && p.y >= 0 && p.y < map.size.1);

    //     if let Ok(space) = (&mut vspace).get(id) {
    //         for vis in vs.visible_tiles.iter() {
    //             let idx = map.xy_idx(vis.to_xy());
    //             space.tiles.insert(idx, (map.tiles[idx], map.tile_content[idx].clone()));
    //         }
    //     }
    //     // }
    // }
}
