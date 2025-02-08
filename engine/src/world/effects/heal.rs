use shipyard::{Get, ViewMut};

use super::*;
use crate::world::components::PhysicalStats;

pub fn heal(store: &AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Heal { amount, target } = &effect.effect_type {
        store.run(|mut stats: ViewMut<PhysicalStats>| {
            for target in get_effected_entities(&store, &target) {
                if let Ok(stats) = (&mut stats).get(target) {
                    stats.hp = i32::min(stats.hp + amount, stats.max_hp);
                }
            }
        });
    }
}
