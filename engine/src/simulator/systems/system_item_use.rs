use crate::ui::colors::{COLOR_UI_3, COLOR_BG};
use crate::simulator::components::{
    AreaOfEffect, PhysicalStats, Confusion, Consumable, DealsDamage, Equippable, Equipped, Inventory,
    Name, ProvidesHealing, WantsToUseItem, GameLog, PlayerID, CausesFire, Equipment,
};
use crate::simulator::effects::add_effect;
use crate::simulator::effects::{EffectType, Targets};
use crate::simulator::map::Map;
use crate::{simulator::components::Position, simulator::systems::system_particle::ParticleBuilder};
use shipyard::{
    AddComponent, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueView, UniqueViewMut, View,
    ViewMut,
};

use super::system_particle::PARTICLE_TIME;

pub fn run_item_use_system(store: AllStoragesViewMut) {
    let mut log = store.borrow::<UniqueViewMut<GameLog>>().unwrap(); //res.get_mut::<GameLog>().unwrap();
    let player_id = store.borrow::<UniqueView<PlayerID>>().ok().unwrap(); //res.get::<EntityId>().unwrap();
    let map = store.borrow::<UniqueView<Map>>().unwrap(); //res.get::<Map>().unwrap();
    let mut p_builder = store.borrow::<UniqueViewMut<ParticleBuilder>>().unwrap(); //res.get_mut::<ParticleBuilder>().unwrap();
    let mut to_remove: Vec<(EntityId, EntityId)> = Vec::new();
    let mut to_remove_wants_use: Vec<EntityId> = Vec::new();
    let mut to_unequip: Vec<(EntityId, Name, EntityId)> = Vec::new();
    let mut to_equip: Vec<(EntityId, Equippable, Name, EntityId)> = Vec::new();

    let mut vwants = store.borrow::<ViewMut<WantsToUseItem>>().unwrap();
    let vaoe = store.borrow::<View<AreaOfEffect>>().unwrap();
    let vstats = store.borrow::<ViewMut<PhysicalStats>>().unwrap();
    let vcausesfire = store.borrow::<View<CausesFire>>().unwrap();
    let vprovideshealing = store.borrow::<View<ProvidesHealing>>().unwrap();
    let vname = store.borrow::<View<Name>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();
    let vdealsdamage = store.borrow::<View<DealsDamage>>().unwrap();
    let vconfusion = store.borrow::<View<Confusion>>().unwrap();
    let vconsumable = store.borrow::<View<Consumable>>().unwrap();
    let vequippable = store.borrow::<View<Equippable>>().unwrap();
    let mut vequipped = store.borrow::<ViewMut<Equipped>>().unwrap();
    let mut vinv = store.borrow::<ViewMut<Inventory>>().unwrap();
    let mut vequipment = store.borrow::<ViewMut<Equipment>>().unwrap();

    for (id, use_item) in vwants.iter().with_id() {
        let mut used_item = false;

        // Find all targets
        let mut targets: Vec<EntityId> = Vec::new();
        let mut target_tiles: Vec<usize> = Vec::new();
        match use_item.target {
            None => targets.push(id),
            Some(t) => {
                match vaoe.get(use_item.item) {
                    Err(_e) => {
                        // Single target
                        let idx = map.xy_idx((t.x, t.y));
                        for entity in map.tile_content[idx].iter() {
                            let stats = vstats.get(*entity);
                            match stats {
                                Err(_e) => {}
                                Ok(_stats) => targets.push(*entity),
                            }
                        }
                    }
                    Ok(aoe) => {
                        // AOE
                        used_item = true;
                        let mut affected_tiles = rltk::field_of_view(t, aoe.radius, &*map);
                        affected_tiles.retain(|p| p.x > 0 && p.x < map.size.0 - 1 && p.y > 0 && p.y < map.size.1 - 1);
                        for pt in affected_tiles.iter() {
                            let idx = map.xy_idx((pt.x, pt.y));
                            target_tiles.push(idx);
                            for entity in map.tile_content[idx].iter() {
                                let stats = vstats.get(*entity);
                                match stats {
                                    Err(_e) => {}
                                    Ok(_stats) => targets.push(*entity),
                                }
                            }
                            p_builder.request(
                                pt.x,
                                pt.y,
                                0.0,
                                0.0,
                                COLOR_UI_3,
                                COLOR_BG,
                                'o',
                                PARTICLE_TIME,
                            )
                        }
                    }
                }
            }
        }

        // Apply fire if it applies fire
        if let Ok(fire) = vcausesfire.get(use_item.item) {
            add_effect(
                Some(id),
                EffectType::Fire {
                    turns: fire.turns,
                    target: Targets::Tiles { tiles: target_tiles },
                },
            );
            used_item = true;
        }

        // Apply heal if it provides healing
        if let Ok(healer) = vprovideshealing.get(use_item.item) {
            for target in targets.iter() {
                let stats = vstats.get(*target);
                match stats {
                    Err(_e) => {}
                    Ok(_stats) => {
                        add_effect(
                            Some(id),
                            EffectType::Heal {
                                amount: healer.heal,
                                target: Targets::Single { target: *target },
                            },
                        );
                        if id == player_id.0 {
                            // todo should this code be in /effects?
                            let name = vname.get(use_item.item).unwrap();
                            log.messages
                                .push(format!("You use the {}, healing {} hp", name.name, healer.heal));
                        }
                        used_item = true;

                        if let Ok(pos) = vpos.get(*target) {
                            for pos in pos.ps.iter() {
                                p_builder.request(
                                    pos.x,
                                    pos.y - 1,
                                    0.0,
                                    -3.0,
                                    COLOR_UI_3,
                                    COLOR_BG,
                                    '♥',
                                    PARTICLE_TIME,
                                )
                            }
                        }
                    }
                }
            }
        }
        to_remove_wants_use.push(id);

        // Apply damage to target if it deals damage
        if let Ok(dd) = vdealsdamage.get(use_item.item) {
            for target in targets.iter() {
                add_effect(
                    Some(id),
                    EffectType::Damage {
                        amount: dd.damage,
                        target: Targets::Single { target: *target },
                    },
                );
                if id == player_id.0 {
                    let monster_name = vname.get(*target).unwrap();
                    let item_name = vname.get(use_item.item).unwrap();
                    log.messages.push(format!(
                        "You use {} on {}, dealing {} hp",
                        item_name.name, monster_name.name, dd.damage
                    ));
                }
                used_item = true;

                if let Ok(pos) = vpos.get(*target) {
                    for pos in pos.ps.iter() {
                        p_builder.request(
                            pos.x,
                            pos.y,
                            0.0,
                            0.0,
                            COLOR_UI_3,
                            COLOR_BG,
                            '‼',
                            PARTICLE_TIME,
                        )
                    }
                }
            }
        }

        // Apply confusion
        if let Ok(confusion) = vconfusion.get(use_item.item){
            for target in targets.iter() {
                add_effect(
                    Some(id),
                    EffectType::Confusion {
                        turns: confusion.turns,
                        target: Targets::Single { target: *target },
                    },
                );
                if id == player_id.0 {
                    let monster_name = vname.get(*target).unwrap();
                    let item_name = vname.get(use_item.item).unwrap();
                    log.messages.push(format!(
                        "You use {} on {}, confusing them",
                        item_name.name, monster_name.name
                    ));
                }
                used_item = true;

                if let Ok(pos) = vpos.get(*target) {
                    for pos in pos.ps.iter() {
                        p_builder.request(
                            pos.x,
                            pos.y,
                            0.0,
                            0.0,
                            COLOR_UI_3,
                            COLOR_BG,
                            '?',
                            PARTICLE_TIME,
                        )
                    }
                }
            }
        }

        // Remove item if it's consumable
        if let Ok(_) = vconsumable.get(use_item.item) {
            if used_item {
                to_remove.push((id, use_item.item));
            }
        }

        // Equip if item is equippable
        if let Ok(equippable) = vequippable.get(use_item.item) {
            let target = targets[0];

            // Unequip already equipped item
            for (id, (equipped, name)) in (&vequipped, &vname).iter().with_id() {
                if equipped.owner == target && equipped.slot == equippable.slot {
                    to_unequip.push((id, name.clone(), target));
                }
            }

            // Actually equip item
            let item_name = (*vname.get(use_item.item).unwrap()).clone();
            to_equip.push((use_item.item, *equippable, item_name, target));
        }
    }

    for (id, item) in to_remove {
        if let Ok(inv) = (&mut vinv).get(id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == item) {
                inv.items.remove(pos);
            }
        }

        add_effect(None, EffectType::Delete { entity: item });
    }

    for id in to_remove_wants_use {
        vwants.remove(id);
    }

    for (id, name, target) in to_unequip {
        vequipped.remove(id);
        if let Ok(inv) = (&mut vinv).get(target) {
            inv.items.push(id)
        } else {
            dbg!("warning: entity unequipped item without inventory");
        }

        if let Ok(equipment) = (&mut vequipment).get(target) {
            equipment.unequip(id);
        }

        if target == player_id.0 {
            log.messages.push(format!("You unequip your {}", name.name));
        }
    }

    for (item, equippable, name, target) in to_equip {
        if let Ok(inv) = (&mut vinv).get(target) {
            inv.items.retain(|&eid| eid != item); // remove item from inventory
        }

        vequipped.add_component_unchecked(
            item,
            Equipped {
                owner: target,
                slot: equippable.slot,
            },
        );

        if let Ok(equipment) = (&mut vequipment).get(target) {
            equipment.equip(item, equippable.slot);
        }

        if target == player_id.0 {
            log.messages.push(format!("You equip your {}", name.name));
        }
    }
}
