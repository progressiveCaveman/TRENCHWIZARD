use shipyard::{AddComponent, Get, UniqueViewMut, ViewMut};

use super::*;
use crate::world::components::PhysicalStats;
use crate::world::components::GameLog;

pub fn inflict_damage(store: &mut AllStoragesViewMut, damage: &EffectSpawner) {
    let mut log = store.borrow::<UniqueViewMut<GameLog>>().unwrap();

    if let EffectType::Damage { amount, target } = &damage.effect_type {
        if let Ok(mut vs) = store.borrow::<ViewMut<PhysicalStats>>() {
            for target in get_effected_entities(&store, &target) {
                match (&vs).get(target) {
                    Ok(stats) => {
                        let mut stats = stats.clone();
                        stats.hp -= amount;
                        vs.add_component_unchecked(target, stats);
                    }
                    Err(_e) => {
                        log.messages.push(format!("Damage failed!!"));
                    }
                }
            }
        }
    }
}
