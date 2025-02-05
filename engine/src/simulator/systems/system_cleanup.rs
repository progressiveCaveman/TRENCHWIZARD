use crate::components::{PhysicalStats, Equipped, Inventory, Name, Player, GameLog};
use crate::simulator::effects::{add_effect, EffectType};
use shipyard::{Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut};

pub fn run_cleanup_system(
    mut log: UniqueViewMut<GameLog>,
    vstats: View<PhysicalStats>,
    vplayer: View<Player>,
    vname: View<Name>,
    mut vinv: ViewMut<Inventory>,
    mut vequip: ViewMut<Equipped>,
) {
    for (id, stats) in (&vstats).iter().with_id() {
        if stats.hp <= 0 {
            match vplayer.get(id) {
                Err(_) => {
                    // not a player
                    if let Ok(inv) = (&mut vinv).get(id) {
                        for e in inv.items.iter() {
                            vequip.remove(*e);
                        }

                        inv.items.clear();
                    }

                    add_effect(None, EffectType::Delete { entity: id });

                    if let Ok(name) = vname.get(id) {
                        log.messages.push(format!("{} is dead", &name.name));
                    }
                }
                Ok(_p) => {
                    // game over handled in main loop
                }
            }
        }
    }
}
