mod system_drop_item;
use shipyard::{IntoIter, IntoWithId, Remove, View, ViewMut};
pub use system_drop_item::run_drop_item_system;

mod system_item_use;
pub use system_item_use::run_item_use_system;

mod system_unequip_item;
pub use system_unequip_item::run_unequip_item_system;

use crate::ai::decisions::{Intent, Target, Task};
use crate::components::{Inventory, WantsToPickupItem};
use crate::effects::{add_effect, EffectType};

pub fn run_inventory_system(vinv: View<Inventory>, vwants: View<WantsToPickupItem>, mut vintent: ViewMut<Intent>) {
    let to_remove_intent = vec![];

    for (id, (_, wants_pickup)) in (&vinv, &vwants).iter().with_id() {
        add_effect(
            Some(id),
            EffectType::PickUp {
                entity: wants_pickup.item,
            },
        );
    }

    for (id, (_, intent)) in (&vinv, &vintent).iter().with_id() {
        if intent.task == Task::PickUpItem {
            if let Target::ENTITY(e) = intent.target[0] {
                add_effect(Some(id), EffectType::PickUp { entity: e });
            }
        }
        if intent.task == Task::DepositItemToInventory {
            if let Target::ENTITY(item) = intent.target[0] {
                if let Target::ENTITY(target) = intent.target[1] {
                    // TODO this looks like a race condition
                    add_effect(Some(id), EffectType::Drop { entity: item });
                    add_effect(Some(target), EffectType::PickUp { entity: item });
                }
            }
        }
    }

    for id in to_remove_intent {
        vintent.remove(id);
    }
}
