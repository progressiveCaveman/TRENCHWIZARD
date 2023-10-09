use crate::components::{
    AreaOfEffect, CombatStats, Confusion, Consumable, DealsDamage, Equippable, Equipped, Fire, InBackpack, Inventory,
    Name, ProvidesHealing, WantsToUseItem,
};
use crate::effects::add_effect;
use crate::effects::{EffectType, Targets};
use crate::map::Map;
use crate::palette::Palette;
use crate::uniques::PlayerID;
use crate::{components::Position, systems::system_particle::ParticleBuilder, uniques::GameLog};
use shipyard::{
    AddComponent, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueView, UniqueViewMut, View,
    ViewMut,
};

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
    let vstats = store.borrow::<ViewMut<CombatStats>>().unwrap();
    let vfire = store.borrow::<View<Fire>>().unwrap();
    let vprovideshealing = store.borrow::<View<ProvidesHealing>>().unwrap();
    let vname = store.borrow::<View<Name>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();
    let vdealsdamage = store.borrow::<View<DealsDamage>>().unwrap();
    let vconfusion = store.borrow::<View<Confusion>>().unwrap();
    let vconsumable = store.borrow::<View<Consumable>>().unwrap();
    let vequippable = store.borrow::<View<Equippable>>().unwrap();
    let mut vequipped = store.borrow::<ViewMut<Equipped>>().unwrap();
    let mut vinv = store.borrow::<ViewMut<Inventory>>().unwrap();
    let mut vinbackpack = store.borrow::<ViewMut<InBackpack>>().unwrap();

    for (id, use_item) in vwants.iter().with_id() {
        let mut used_item = true;

        // Find all targets
        let mut targets: Vec<EntityId> = Vec::new();
        let mut target_tiles: Vec<usize> = Vec::new();
        match use_item.target {
            None => targets.push(player_id.0),
            Some(t) => {
                match vaoe.get(use_item.item) {
                    Err(_e) => {
                        // Single target
                        let idx = map.xy_idx(t.x, t.y);
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
                        let mut affected_tiles = rltk::field_of_view(t, aoe.radius, &*map);
                        affected_tiles.retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
                        for pt in affected_tiles.iter() {
                            let idx = map.xy_idx(pt.x, pt.y);
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
                                Palette::COLOR_3,
                                Palette::MAIN_BG,
                                rltk::to_cp437('o'),
                                250.0,
                            )
                        }
                    }
                }
            }
        }

        // Apply fire if it applies fire
        let item_fires = vfire.get(use_item.item);
        match item_fires {
            Err(_e) => {}
            Ok(fire) => {
                add_effect(
                    Some(id),
                    EffectType::Fire {
                        turns: fire.turns,
                        target: Targets::Tiles { tiles: target_tiles },
                    },
                );
                used_item = true;
            }
        }

        // Apply heal if it provides healing
        let item_heals = vprovideshealing.get(use_item.item);
        match item_heals {
            Err(_e) => {}
            Ok(healer) => {
                used_item = false;
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
                                        pos.y,
                                        0.0,
                                        -3.0,
                                        Palette::COLOR_3,
                                        Palette::MAIN_BG,
                                        rltk::to_cp437('♥'),
                                        1000.0,
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
        to_remove_wants_use.push(id);

        // Apply damage to target if it deals damage
        let deals_damage = vdealsdamage.get(use_item.item);
        match deals_damage {
            Err(_e) => {}
            Ok(dd) => {
                used_item = false;
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
                                Palette::COLOR_4,
                                Palette::MAIN_BG,
                                rltk::to_cp437('‼'),
                                250.0,
                            )
                        }
                    }
                }
            }
        }

        // Apply confusion
        let confusion = vconfusion.get(use_item.item);
        match confusion {
            Err(_e) => {}
            Ok(confusion) => {
                used_item = false;
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
                                Palette::COLOR_3,
                                Palette::MAIN_BG,
                                rltk::to_cp437('?'),
                                300.0,
                            )
                        }
                    }
                }
            }
        }

        // Remove item if it's consumable
        let consumable = vconsumable.get(use_item.item);
        match consumable {
            Err(_e) => {}
            Ok(_) => {
                if used_item {
                    to_remove.push((id, use_item.item));
                }
            }
        }

        // Equip if item is equippable
        let equippable = vequippable.get(use_item.item);
        match equippable {
            Err(_e) => {}
            Ok(equippable) => {
                let target = targets[0];

                // Unequip already equipped item
                for (id, (equipped, name)) in (&vequipped, &vname).iter().with_id() {
                    //world.query::<(&Equipped, &Name)>().iter() {
                    if equipped.owner == target && equipped.slot == equippable.slot {
                        to_unequip.push((id, name.clone(), target));
                    }
                }

                // Actually equip item
                let item_name = (*vname.get(use_item.item).unwrap()).clone();
                to_equip.push((use_item.item, *equippable, item_name, target));
            }
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
        vinbackpack.add_component_unchecked(id, InBackpack { owner: target });
        if target == player_id.0 {
            log.messages.push(format!("You unequip your {}", name.name));
        }
    }

    for (id, equippable, name, target) in to_equip {
        vinbackpack.remove(id);
        vequipped.add_component_unchecked(
            id,
            Equipped {
                owner: target,
                slot: equippable.slot,
            },
        );
        if target == player_id.0 {
            log.messages.push(format!("You equip your {}", name.name));
        }
    }
}
