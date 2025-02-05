use crate::simulator::components::{Position, AddsGas, RNG, RemovesGas};
use crate::map::Map;
use crate::tiles::STABLE_GAS_AMOUNT;
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
                map.gases[idx].0 = vec![];
            }
        }
    }

    let mut new_gases = map.gases.clone();// todo clone slow

    // run gas dissipation
    for (idx, _) in map.gases.iter().enumerate() {
        let point = map.idx_point(idx);
        let gas_amt = map.gases[idx].0.len() as f32;

        if gas_amt == 0.0 {
            continue;
        }
        // let steamcount = map.gas_count(idx, GasType::Steam);

        // if steamcount > 0 {
            let mut flow_target = 0;
            let mut flow_best = 0.0;
            for &n in get_neighbors(point).iter().filter(|p| map.in_bounds(p.to_xy()) && !map.is_wall(p.x, p.y)) {
                let nidx = map.point_idx(n);

                // let dist = 1.0;//map.get_pathing_distance(*lastflow, nidx);
                // let air_amount = map.gas_count(nidx, GasType::Air) as f32 / MAX_GAS as f32;
                // let none_amount = map.gas_count(nidx, GasType::None);
                let pressure = STABLE_GAS_AMOUNT as f32 / map.gases[nidx].0.len() as f32;

                let score = pressure * rng.0.roll_dice(1, 5) as f32 / 5.0;
                // let mut score = air_amount * dist * rng.0.roll_dice(1, 5) as f32 / 10.0;
                
                // if none_amount > 0 {
                //     score *= 10.0;
                // }

                // for gas in map.gases[nidx].0.iter() {
                //     if *gas == GasType::Air {
                //         if score > flow_best {
                //             flow_target = nidx;
                //             flow_best = score;
                //         }
                //         break;
                //     }
                // }

                if score > flow_best {
                    flow_target = nidx;
                    flow_best = score;
                }
            }
    
            if flow_target != 0 && gas_amt + rng.0.roll_dice(1, gas_amt as i32) as f32 > STABLE_GAS_AMOUNT as f32 {
                let randsrc = rng.0.roll_dice(1, gas_amt as i32) as usize - 1;
                let gas = new_gases[idx].0.remove(randsrc);
                new_gases[flow_target].0.push(gas);

                // for gas in new_gases[idx].0.iter_mut() {
                //     if *gas == GasType::Steam {
                //         *gas = GasType::Air;
                //         break;
                //     }
                // }

                // new_gases[flow_target].1 = idx; // set flow
                // for gas in new_gases[flow_target].0.iter_mut() {
                //     if *gas == GasType::Air {
                //         *gas = GasType::Steam;
                //         break;
                //     }
                // }
            }
        // }
    }

    // Run AddsGas components
    for (_, (pos, addsgas)) in (&vpos, &mut vaddsgas).iter().with_id() {
        let idx = map.point_idx(pos.ps[0]);
        new_gases[idx].0.push(addsgas.gas);
        // for gas in new_gases[idx].0.iter_mut() {
        //     if *gas == GasType::Air {
        //         *gas = addsgas.gas;
        //         break;
        //     }
        // }
    }

    // Run removes gas components
    for (_, (pos, _)) in (&vpos, &mut vremovesgas).iter().with_id() {
        let idx = map.point_idx(pos.ps[0]);
        let gas_amt = new_gases[idx].0.len();
        if gas_amt > 0 {
            let gasidx = rng.0.roll_dice(1, gas_amt as i32) as usize - 1;
            new_gases[idx].0.remove(gasidx);
        }
        // for gas in new_gases[idx].0.iter_mut() {
        //     if *gas != GasType::Air {
        //         *gas = GasType::Air;
        //         break;
        //     }
        // }
    }

    //replace old gases
    map.gases = new_gases;
}
