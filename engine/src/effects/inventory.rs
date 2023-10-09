use shipyard::{AddComponent, Get, Remove, UniqueView, UniqueViewMut, View, ViewMut};

use super::*;
use crate::{
    components::{Equipped, InBackpack, Inventory, Name, Position, WantsToPickupItem},
    uniques::GameLog,
    uniques::PlayerID,
};

pub fn pick_up(store: &AllStoragesViewMut, effect: &EffectSpawner) {
    let mut vpos = store.borrow::<ViewMut<Position>>().unwrap();
    let vname = store.borrow::<View<Name>>().unwrap();
    let mut vinv = store.borrow::<ViewMut<Inventory>>().unwrap();
    let mut vwantspickup = store.borrow::<ViewMut<WantsToPickupItem>>().unwrap();
    let mut vpacks = store.borrow::<ViewMut<InBackpack>>().unwrap();

    if let (Some(id), EffectType::PickUp { entity: target }) = (effect.creator, &effect.effect_type) {
        let mut log = store.borrow::<UniqueViewMut<GameLog>>().unwrap();
        let player_id = store.borrow::<UniqueView<PlayerID>>().unwrap().0;

        if let Err(_) = vpos.get(id) {
            dbg!("Entity doesn't have a position");
            return;
        }

        if let Err(_) = vpos.get(*target) {
            // dbg!("Entity doesn't have a position");
            return;
        }

        if let Ok(name) = vname.get(id) {
            if let Ok(inv) = (&mut vinv).get(id) {
                inv.items.push(*target);

                let mut entities: Vec<EntityId> = vec![];
                for e1 in inv.items.iter() {
                    let mut dup = false;
                    for e2 in entities.iter() {
                        if e2 == e1 {
                            dup = true;
                            println!("ERROR: Duplicate item in {:?} inventory", &name.name);
                            dbg!(42);
                            // return;
                        }
                    }
                    if !dup {
                        entities.push(*e1);
                    }
                }
            } else {
                dbg!("Entity has no inventory");
            }
        }

        let _res = vpos.remove(*target);
        let _r = vpacks.add_component_unchecked(*target, InBackpack { owner: id });

        if id == player_id {
            let name = vname.get(*target).unwrap();
            log.messages.push(format!("You pick up the {}", name.name));
        }

        let _re = vwantspickup.remove(id);
    }
}

pub fn drop_item(store: &AllStoragesViewMut, effect: &EffectSpawner) {
    let mut vpos = store.borrow::<ViewMut<Position>>().unwrap();
    let mut vinv = store.borrow::<ViewMut<Inventory>>().unwrap();
    let mut vpack = store.borrow::<ViewMut<InBackpack>>().unwrap();
    let mut vequipped = store.borrow::<ViewMut<Equipped>>().unwrap();

    if let (Some(id), EffectType::Drop { entity: target }) = (effect.creator, &effect.effect_type) {
        let pos = if let Ok(p) = vpos.get(id) {
            p.any_point()
        } else {
            unreachable!()
        };
        if let Ok(inv) = (&mut vinv).get(id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == *target) {
                inv.items.remove(pos);
            }
        }

        vpack.remove(id);
        vequipped.remove(id);
        vpos.add_component_unchecked(*target, Position { ps: vec![pos] });
    }
}
