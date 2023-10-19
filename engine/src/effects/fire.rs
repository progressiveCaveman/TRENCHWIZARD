use std::cmp;

use shipyard::{UniqueViewMut, ViewMut};

use super::*;
use crate::{components::Fire, map::Map};

// pub fn inflict_fire_tile(gs: &mut State, effect: &EffectSpawner, tile_idx: usize) {

//     if let EffectType::Fire { turns } = effect.effect_type {
//         if map.is_flammable(tile_idx) {
//             map.fire_turns[tile_idx] += turns;
//         }
//     }
// }

pub fn inflict_fire(store: &mut AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Fire { turns, target } = &effect.effect_type {
        for target in get_effected_entities(&store, &target) {
            let mut to_add_fire = vec![];
            store.run(|vfire: ViewMut<Fire>| {
                if let Ok(fire) = vfire.get(target) {
                    to_add_fire.push((
                        target,
                        Fire {
                            turns: cmp::max(fire.turns, *turns),
                        },
                    ));
                } else {
                    to_add_fire.push((target, Fire { turns: *turns }));
                }
            });

            for (target, fire) in to_add_fire {
                store.add_component(target, fire);
            }
        }

        let mut map = store.borrow::<UniqueViewMut<Map>>().unwrap();
        for tile_idx in get_effected_tiles(&store, &target) {
            if map.is_flammable(tile_idx) {
                map.fire_turns[tile_idx] += turns;
            }
        }
    }
}
