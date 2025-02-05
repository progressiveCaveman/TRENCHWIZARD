use super::*;
use crate::simulator::components::Confusion;

pub fn inflict_confusion(store: &mut AllStoragesViewMut, confusion: &EffectSpawner) {
    if let EffectType::Confusion { turns, target } = &confusion.effect_type {
        for entity in get_effected_entities(&store, &target) {
            store.add_component(entity, Confusion { turns: *turns });
        }
    }
}
