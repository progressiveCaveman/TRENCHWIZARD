use std::cmp;

use shipyard::{UniqueViewMut, ViewMut, EntitiesViewMut};

use super::*;
use crate::{simulator::map::Map, simulator::components::OnFire};

pub fn inflict_fire(store: &mut AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Fire { turns, target } = &effect.effect_type {
        let mut to_add_fire = vec![];
        for target in get_effected_entities(&store, &target) {
            store.run(|vfire: ViewMut<OnFire>| {
                if let Ok(fire) = vfire.get(target) {
                    to_add_fire.push((
                        target,
                        OnFire {
                            turns: cmp::min(fire.turns, *turns),
                        },
                    ));
                } else {
                    to_add_fire.push((target, OnFire { turns: *turns }));
                }
            });
        }

        store.run(|mut vfire: ViewMut<OnFire>, entities: EntitiesViewMut | {
            for (target, fire) in to_add_fire {
                if entities.is_alive(target) {
                    entities.add_component(target, &mut vfire, fire);
                }
            }
        });


        for tile_idx in get_effected_tiles(&store, &target) {
            let mut map = store.borrow::<UniqueViewMut<Map>>().unwrap();
            if map.is_flammable(tile_idx) {
                map.fire_turns[tile_idx] += turns;
            }
        }
    }
}