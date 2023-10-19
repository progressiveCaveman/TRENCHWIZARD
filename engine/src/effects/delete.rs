use super::*;

pub fn delete(store: &mut AllStoragesViewMut, effect: &EffectSpawner) {
    if let EffectType::Delete { entity } = effect.effect_type {
        store.delete_entity(entity);
    }
}
