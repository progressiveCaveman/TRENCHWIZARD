use shipyard::UniqueViewMut;

use super::*;
use crate::entity_factory::spawn_entity_type;

pub fn spawn(store: &mut AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Spawn { etype, target } = &effect.effect_type {
        let mut spawn_targets = vec![];
        for tile_idx in get_effected_tiles(&store, &target) {
            let map = store.borrow::<UniqueViewMut<Map>>().unwrap();
            spawn_targets.push(map.idx_xy(tile_idx))
    
            // let mut map = store.borrow::<UniqueViewMut<Map>>().unwrap();
            // if map.is_flammable(tile_idx) {
            //     map.fire_turns[tile_idx] += turns;
            // }
        }
        for t in spawn_targets.iter() {
            spawn_entity_type(store, *etype, *t);
        }
    }

    // if let EffectType::Heal { amount, target } = &effect.effect_type {
    //     store.run(|mut stats: ViewMut<PhysicalStats>| {
    //         for target in get_effected_entities(&store, &target) {
    //             if let Ok(stats) = (&mut stats).get(target) {
    //                 stats.hp = i32::min(stats.hp + amount, stats.max_hp);
    //             }
    //         }
    //     });
    // }
}
