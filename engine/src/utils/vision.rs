use shipyard::{EntityId, View, Get, AllStoragesViewMut};

use crate::components::{Vision, Position};

pub fn vision_contains(store: &AllStoragesViewMut, vision: Vision, id: EntityId) -> bool{
    return store.run(|vpos: View<Position>| {
        if let Ok(pos) = vpos.get(id) {
            for pos in pos.ps.iter() {
                for p in vision.visible_tiles.iter() {
                    if pos == p {
                        return true
                    }
                }
            }
        }

        return false
    })
}