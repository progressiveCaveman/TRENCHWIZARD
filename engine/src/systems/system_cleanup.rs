use crate::components::{CombatStats, Equipped, InBackpack, Inventory, Name, Player, GameLog};
use crate::effects::{add_effect, EffectType};
use shipyard::{Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut};

pub fn run_cleanup_system(
    mut log: UniqueViewMut<GameLog>,
    vstats: View<CombatStats>,
    vinv: View<Inventory>,
    vplayer: View<Player>,
    vname: View<Name>,
    mut vpack: ViewMut<InBackpack>,
    mut vequip: ViewMut<Equipped>,
) {
    for (id, stats) in (&vstats).iter().with_id() {
        if stats.hp <= 0 {
            match vplayer.get(id) {
                Err(_) => {
                    // not a player
                    if let Ok(inv) = vinv.get(id) {
                        for e in inv.items.iter() {
                            vpack.remove(*e);
                            vequip.remove(*e);
                        }
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
