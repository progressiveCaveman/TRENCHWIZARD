use crate::components::{CombatStats, Equipped, InBackpack, Inventory, Name, Player};
use crate::effects::{add_effect, EffectType};
use crate::uniques::GameLog;
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
            let player = vplayer.get(id);
            let name = vname.get(id);
            match player {
                Err(_) => {
                    // not a player
                    if let Ok(inv) = vinv.get(id) {
                        for e in inv.items.iter() {
                            vpack.remove(*e);
                            vequip.remove(*e);
                        }
                    }

                    add_effect(None, EffectType::Delete { entity: id });

                    if let Ok(name) = name {
                        log.messages.push(format!("{} is dead", &name.name));
                    }
                }
                Ok(_p) => {
                    todo!("Game over");
                    // *runstate = RunState::GameOver;
                }
            }
        }
    }
}
