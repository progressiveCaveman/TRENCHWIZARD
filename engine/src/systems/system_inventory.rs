use shipyard::{IntoIter, IntoWithId, Remove, View, ViewMut, UniqueViewMut, EntityId, Get};

use crate::ai::decisions::{Intent, Task};
use crate::components::{Inventory, WantsToPickupItem, GameLog, Player, WantsToUnequipItem, Equipped, Name};
use crate::effects::{add_effect, EffectType};
use crate::utils::Target;
use crate::components::WantsToDropItem;

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

pub fn run_unequip_item_system(
    mut log: UniqueViewMut<GameLog>,
    vplayer: View<Player>,
    mut vinv: ViewMut<Inventory>,
    mut vwants: ViewMut<WantsToUnequipItem>,
    mut vequip: ViewMut<Equipped>,
    vname: View<Name>,
) {
    let mut to_remove_wants: Vec<EntityId> = vec![];
    let mut to_unequip = vec![];

    for (id, (_, wants_unequip)) in (&vinv, &vwants).iter().with_id() {
        to_remove_wants.push(id);
        to_unequip.push((id, wants_unequip.item));

        if let Ok(_) = vplayer.get(id) {
            if let Ok(item_name) = vname.get(wants_unequip.item) {
                log.messages.push(format!("You unequip the {}", item_name.name));
            }
        }
    }

    for (entity, item) in to_unequip.iter() {
        vwants.remove(*entity);
        vequip.remove(*item);
        if let Ok(inv) = (&mut vinv).get(*entity){
            inv.items.push(*item);
        }
    }
}

pub fn run_drop_item_system(
    mut log: UniqueViewMut<GameLog>,
    vplayer: View<Player>,
    mut vwants: ViewMut<WantsToDropItem>,
    vname: View<Name>,
) {
    let mut to_drop: Vec<(EntityId, EntityId)> = Vec::new();

    for (id, wants_drop) in vwants.iter().with_id() {
        to_drop.push((id, wants_drop.item));

        if let Ok(_) = vplayer.get(id) {
            if let Ok(item_name) = vname.get(wants_drop.item) {
                log.messages.push(format!("You drop the {}", item_name.name));
            }
        }
    }

    for (id, item) in to_drop.iter() {
        vwants.remove(*id);
        add_effect(Some(*id), EffectType::Drop { entity: *item });
    }
}
