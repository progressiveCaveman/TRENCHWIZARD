use shipyard::{AddComponent, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut};

use crate::{
    components::{Equipped, InBackpack, Inventory, Name, Player, WantsToUnequipItem},
    uniques::GameLog,
};

pub fn run_unequip_item_system(
    mut log: UniqueViewMut<GameLog>,
    vplayer: View<Player>,
    vinv: View<Inventory>,
    mut vwants: ViewMut<WantsToUnequipItem>,
    mut vequip: ViewMut<Equipped>,
    mut vbackpack: ViewMut<InBackpack>,
    vname: View<Name>,
) {
    let mut to_remove_wants: Vec<EntityId> = vec![];

    for (id, (_, wants_unequip)) in (&vinv, &vwants).iter().with_id() {
        to_remove_wants.push(id);
        vequip.remove(wants_unequip.item);
        vbackpack.add_component_unchecked(wants_unequip.item, InBackpack { owner: id });

        if let Ok(_) = vplayer.get(id) {
            if let Ok(item_name) = vname.get(wants_unequip.item) {
                log.messages.push(format!("You unequip the {}", item_name.name));
            }
        }
    }

    for e in to_remove_wants.iter() {
        vwants.remove(*e);
    }
}
