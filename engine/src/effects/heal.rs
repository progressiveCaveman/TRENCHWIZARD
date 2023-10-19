use shipyard::{Get, ViewMut};

use super::*;
use crate::components::CombatStats;

pub fn heal(store: &AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Heal { amount, target } = &effect.effect_type {
        store.run(|mut stats: ViewMut<CombatStats>| {
            for target in get_effected_entities(&store, &target) {
                if let Ok(mut stats) = (&mut stats).get(target) {
                    stats.hp = i32::min(stats.hp + amount, stats.max_hp);
                }
            }
        });
    }
}
