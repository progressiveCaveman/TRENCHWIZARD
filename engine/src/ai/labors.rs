use rltk::Point;
use shipyard::{AllStorages, EntityId, Get, UniqueView, View};

use crate::{
    components::{
        Actor, ActorType, FishCleaner, Inventory, Item, ItemType, LumberMill, Position, SpatialKnowledge, Tree, Vision, Turn,
    },
    map::Map, utils::Target, tiles::TileType,
};

use super::decisions::{Action, Consideration, ConsiderationParam, Intent, ResponseCurveType, Task, AI};

#[derive(Clone, Debug, PartialEq)]
pub enum AIBehaviors {
    GatherWood,
    GatherFish,
    AttackEnemies,
    Confused,
    Wander,
}

pub fn get_action(store: &AllStorages, id: EntityId) -> Action {
    let vactor = store.borrow::<View<Actor>>().unwrap();
    let turn = store.borrow::<UniqueView<Turn>>().unwrap();

    let mut potential_actions: Vec<Action> = vec![];

    potential_actions.push(Action {
        intent: Intent {
            name: "idle".to_string(),
            task: Task::Idle,
            target: vec![],
            turn: *turn,
        },
        cons: vec![Consideration::new(
            "baseline".to_string(),
            1.0,
            ConsiderationParam::new_const(0.1),
        )],
        priority: 1.0,
    });

    if let Ok(actor) = vactor.get(id) {
        for b in actor.behaviors.iter() {
            match b {
                AIBehaviors::GatherWood => potential_actions.append(&mut get_gather_wood_actions(&store, id)),
                AIBehaviors::GatherFish => potential_actions.append(&mut get_gather_fish_actions(&store, id)),
                AIBehaviors::AttackEnemies => potential_actions.append(&mut get_attack_actions(&store, id)),
                _ => {} // AIBehaviors::Wander => ,
            }
        }
    }

    return AI::choose_action(potential_actions);
}

pub fn get_gather_wood_actions(store: &AllStorages, id: EntityId) -> Vec<Action> {
    let turn = store.borrow::<UniqueView<Turn>>().unwrap();
    let map = store.borrow::<UniqueView<Map>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();
    let vitem = store.borrow::<View<Item>>().unwrap();
    let vtree = store.borrow::<View<Tree>>().unwrap();
    let vlm = store.borrow::<View<LumberMill>>().unwrap();
    let vspace = store.borrow::<View<SpatialKnowledge>>().unwrap();
    let vinv = store.borrow::<View<Inventory>>().unwrap();

    let pos = if let Ok(pos) = vpos.get(id) {
        pos
    } else {
        return vec![];
    };
    let space = if let Ok(pos) = vspace.get(id) {
        pos
    } else {
        return vec![];
    };
    let inv = if let Ok(pos) = vinv.get(id) {
        pos
    } else {
        return vec![];
    };

    let pos = pos.ps[0];

    let has_inventory_space = inv.capacity > inv.items.len() as i32;

    let mut logs_in_inv = 0;
    let mut inventory_log: EntityId = id; // initialization is messy here but correct as long as logs_in_inv > 0
    for e in inv.items.iter() {
        if let Ok(item) = vitem.get(*e) {
            if item.typ == ItemType::Log {
                logs_in_inv += 1;
                inventory_log = *e;
            }
        }
    }

    // populate all our info
    let mut trees: Vec<EntityId> = vec![];
    let mut logs: Vec<EntityId> = vec![];
    let mut lumber_mills: Vec<EntityId> = vec![];
    for (_, entities) in space.tiles.values() {
        for e in entities.iter() {
            if let Ok(_) = vtree.get(*e) {
                trees.push(*e);
            }
            if let Ok(item) = vitem.get(*e) {
                if item.typ == ItemType::Log {
                    logs.push(*e);
                }
            }
            if let Ok(_) = vlm.get(*e) {
                if !lumber_mills.contains(e) {
                    //multitile
                    lumber_mills.push(*e);
                }
            }
        }
    }

    let mut potential_actions: Vec<Action> = vec![];

    // for each tree found
    for tree in trees {
        if has_inventory_space {
            potential_actions.push(Action {
                intent: Intent {
                    name: "go to tree".to_string(),
                    task: Task::MoveTo,
                    target: vec![Target::from(tree)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(tree)),
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: -1.0 / 100.0,
                            k: 1.0,
                            c: 1.0,
                            b: 1.0,
                        },
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(),
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam {
                    //         t: todo!(),
                    //         m: 0.0,
                    //         k: 0.0,
                    //         c: 0.0,
                    //         b: 0.0
                    //     }
                    // )
                ],
                priority: 1.0,
            });

            potential_actions.push(Action {
                intent: Intent {
                    name: "chop tree".to_string(),
                    task: Task::Destroy,
                    target: vec![Target::from(tree)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(tree)),
                        ConsiderationParam {
                            t: ResponseCurveType::LessThan,
                            m: 2.,
                            k: 1.0,
                            c: 1.0,
                            b: 1.0,
                        },
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(),
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam {
                    //         t: todo!(),
                    //         m: 0.0,
                    //         k: 0.0,
                    //         c: 0.0,
                    //         b: 0.0
                    //     }
                    // )
                ],
                priority: 2.0,
            });
        }
    }

    // for each wood found
    for log in logs.iter() {
        if has_inventory_space {
            potential_actions.push(Action {
                intent: Intent {
                    name: "pick up wood".to_string(),
                    task: Task::PickUpItem,
                    target: vec![Target::from(*log)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(*log)),
                        ConsiderationParam {
                            t: ResponseCurveType::LessThan,
                            m: 2.,
                            k: 1.0,
                            c: 0.0,
                            b: 1.0,
                        },
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(),
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam {
                    //         t: todo!(),
                    //         m: 0.0,
                    //         k: 0.0,
                    //         c: 0.0,
                    //         b: 0.0
                    //     }
                    // )
                ],
                priority: 1.0,
            });
        }
    }

    // if wood in inventory
    // for each LumberMill
    for lm in lumber_mills {
        let lminv = if let Ok(inv) = vinv.get(lm) {
            inv
        } else {
            continue;
        };
        let lminv_count = lminv.count_type(&vitem, ItemType::Log) as f32;

        if logs_in_inv > 0 {
            potential_actions.push(Action {
                intent: Intent {
                    name: "move to lumber mill".to_string(),
                    task: Task::MoveTo,
                    target: vec![Target::from(lm)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(lm)),
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. - 1. / 20.,
                            k: 1.0,
                            c: 1.0,
                            b: 0.0,
                        },
                    ),
                    // Consideration::new(
                    //     "logs in stockpile".to_string(),
                    //     lminv_count,
                    //     ConsiderationParam {
                    //         t: ResponseCurveType::Linear,
                    //         m: -1. / 50.0,
                    //         k: 1.0,
                    //         c: 0.0,
                    //         b: 1.0,
                    //     },
                    // ),
                    Consideration::new(
                        "logs in iventory".to_string(),
                        logs_in_inv as f32,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. / 5.0,
                            k: 1.0,
                            c: 0.0,
                            b: 0.0,
                        },
                    ),
                ],
                priority: 1.0,
            });

            potential_actions.push(Action {
                intent: Intent {
                    name: "deposit logs at lumber mill".to_string(),
                    task: Task::DepositItemToInventory,
                    target: vec![Target::from(inventory_log), Target::from(lm)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance to lm".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(lm)),
                        ConsiderationParam {
                            t: ResponseCurveType::LessThan,
                            m: 2.,
                            k: 2.0,
                            c: 1.0,
                            b: 0.0,
                        },
                    ),
                    // Consideration::new(
                    //     "logs in stockpile".to_string(),
                    //     lminv_count,
                    //     ConsiderationParam {
                    //         t: ResponseCurveType::Linear,
                    //         m: -1. / 50.0,
                    //         k: 1.0,
                    //         c: 0.0,
                    //         b: 1.0,
                    //     },
                    // ),
                    Consideration::new(
                        "logs in iventory".to_string(),
                        logs_in_inv as f32,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. / 5.0,
                            k: 1.0,
                            c: 0.0,
                            b: 0.0,
                        },
                    ),
                ],
                priority: 2.0,
            });
        }
    }

    // wander action
    potential_actions.push(Action {
        intent: Intent {
            name: "explore".to_string(),
            task: Task::Explore,
            target: vec![],
            turn: *turn,
        },
        cons: vec![Consideration::new(
            "baseline".to_string(),
            1.0,
            ConsiderationParam::new_const(0.1),
        )],
        priority: 1.0,
    });

    potential_actions
}

pub fn get_gather_fish_actions(store: &AllStorages, id: EntityId) -> Vec<Action> {
    let turn = store.borrow::<UniqueView<Turn>>().unwrap();
    let map = store.borrow::<UniqueView<Map>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();
    let vitem = store.borrow::<View<Item>>().unwrap();
    let vactors = store.borrow::<View<Actor>>().unwrap(); // Used to find fish
    let vfishery = store.borrow::<View<FishCleaner>>().unwrap();
    let vspace = store.borrow::<View<SpatialKnowledge>>().unwrap();
    let vinv = store.borrow::<View<Inventory>>().unwrap();

    let pos = if let Ok(pos) = vpos.get(id) {
        pos
    } else {
        return vec![];
    };
    let space = if let Ok(pos) = vspace.get(id) {
        pos
    } else {
        return vec![];
    };
    let inv = if let Ok(pos) = vinv.get(id) {
        pos
    } else {
        return vec![];
    };

    let pos = pos.ps[0];

    let has_inventory_space = inv.capacity > inv.items.len() as i32;

    let mut fish_in_inv = 0;
    let mut inventory_fish: EntityId = id; // initialization is messy here but correct as long as logs_in_inv > 0
    for e in inv.items.iter() {
        if let Ok(actor) = vactors.get(*e) {
            if actor.atype == ActorType::Fish {
                fish_in_inv += 1;
                inventory_fish = *e;
            }
        }
    }

    // populate all our info
    let mut water: Vec<Point> = vec![]; // actually points adjacent to water
    let mut fisheries: Vec<EntityId> = vec![];

    for (idx, (tile, entities)) in space.tiles.iter() {
        if *tile == TileType::Water {
            // todo actually path to water to test if it should be considered?
            let mut apoint = map.idx_point(*idx);
            apoint.y -= 1;
            let aboveidx = map.point_idx(apoint);
            if map.tiles[aboveidx] != TileType::Water {
                water.push(apoint);
            }
        }

        for e in entities.iter() {
            // if let Ok(_) = world.get::<Tree>(*e) {
            //     trees.push(*e);
            // }
            // if let Ok(item) = world.get::<Item>(*e) {
            //     if item.typ == ItemType::Log {
            //         logs.push(*e);
            //     }
            // }
            if let Ok(_) = vfishery.get(*e) {
                if !fisheries.contains(e) {
                    //multitile
                    fisheries.push(*e);
                }
            }
        }
    }

    let mut potential_actions: Vec<Action> = vec![];

    // for each water tile found
    for wp in water {
        if has_inventory_space {
            potential_actions.push(Action {
                intent: Intent {
                    name: "go to water".to_string(),
                    task: Task::MoveTo,
                    target: vec![Target::from(wp)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(wp)),
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: -1.0 / 100.0,
                            k: 1.0,
                            c: 2.0,
                            b: 1.0,
                        },
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(),
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam {
                    //         t: todo!(),
                    //         m: 0.0,
                    //         k: 0.0,
                    //         c: 0.0,
                    //         b: 0.0
                    //     }
                    // )
                ],
                priority: 1.0,
            });

            potential_actions.push(Action {
                intent: Intent {
                    name: "fish at water".to_string(),
                    task: Task::Fish,
                    target: vec![Target::from(wp)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(wp)),
                        ConsiderationParam {
                            t: ResponseCurveType::LessThan,
                            m: 1.,
                            k: 1.0,
                            c: 1.0,
                            b: 1.0,
                        },
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(),
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam {
                    //         t: todo!(),
                    //         m: 0.0,
                    //         k: 0.0,
                    //         c: 0.0,
                    //         b: 0.0
                    //     }
                    // )
                ],
                priority: 2.0,
            });
        }
    }

    // if fish in inventory
    // for each fish cleaner
    for f in fisheries {
        let finv = if let Ok(inv) = vinv.get(f) {
            inv
        } else {
            continue;
        };
        let finv_count = finv.count_type(&vitem, ItemType::Fish) as f32;

        if fish_in_inv > 0 {
            potential_actions.push(Action {
                intent: Intent {
                    name: "move to fishery".to_string(),
                    task: Task::MoveTo,
                    target: vec![Target::from(f)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(f)),
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. - 1. / 20.,
                            k: 1.0,
                            c: 1.0,
                            b: 0.0,
                        },
                    ),
                    Consideration::new(
                        "fish in stockpile".to_string(),
                        finv_count,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: -1. / 50.0,
                            k: 1.0,
                            c: 0.0,
                            b: 1.0,
                        },
                    ),
                    Consideration::new(
                        "fish in iventory".to_string(),
                        fish_in_inv as f32,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. / 5.0,
                            k: 1.0,
                            c: 0.0,
                            b: 0.0,
                        },
                    ),
                ],
                priority: 1.0,
            });

            potential_actions.push(Action {
                intent: Intent {
                    name: "deposit fish at fishery".to_string(),
                    task: Task::DepositItemToInventory,
                    target: vec![Target::from(inventory_fish), Target::from(f)],
                    turn: *turn,
                },
                cons: vec![
                    Consideration::new(
                        "Distance to fishery".to_string(),
                        map.distance(&vpos, Target::from(pos), Target::from(f)),
                        ConsiderationParam {
                            t: ResponseCurveType::LessThan,
                            m: 2.,
                            k: 2.0,
                            c: 1.0,
                            b: 0.0,
                        },
                    ),
                    Consideration::new(
                        "fish in stockpile".to_string(),
                        finv_count,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: -1. / 50.0,
                            k: 1.0,
                            c: 0.0,
                            b: 1.0,
                        },
                    ),
                    Consideration::new(
                        "fish in iventory".to_string(),
                        fish_in_inv as f32,
                        ConsiderationParam {
                            t: ResponseCurveType::Linear,
                            m: 1. / 5.0,
                            k: 1.0,
                            c: 0.0,
                            b: 0.0,
                        },
                    ),
                ],
                priority: 2.0,
            });
        }
    }

    potential_actions
}

pub fn get_attack_actions(store: &AllStorages, id: EntityId) -> Vec<Action> {
    let turn = store.borrow::<UniqueView<Turn>>().unwrap();
    let map = store.borrow::<UniqueView<Map>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();
    let vactors = store.borrow::<View<Actor>>().unwrap(); // Used to find fish
    let vvs = store.borrow::<View<Vision>>().unwrap();

    let pos = if let Ok(pos) = vpos.get(id) {
        pos
    } else {
        return vec![];
    };

    let actor = if let Ok(actor) = vactors.get(id) {
        actor
    } else {
        return vec![];
    };

    let viewshed = if let Ok(vs) = vvs.get(id) {
        vs
    } else {
        return vec![];
    };

    let pos = pos.ps[0];

    // if enemies are present, move toward them or attack
    let mut enemies: Vec<(EntityId, Point)> = vec![];

    for point in viewshed.visible_tiles.iter() {
        let idx = map.point_idx(*point);
        for entity in map.tile_content[idx].iter() {
            if let Ok(eactor) = vactors.get(*entity) {
                if actor.faction != eactor.faction {
                    // TODO need more complex faction relations at some point
                    enemies.push((*entity, *point));
                }
            }
        }
    }

    let mut potential_actions: Vec<Action> = vec![];

    for (_, epoint) in enemies.iter() {
        potential_actions.push(Action {
            intent: Intent {
                name: "go to enemy".to_string(),
                task: Task::MoveTo,
                target: vec![Target::from(*epoint)],
                turn: *turn,
            },
            cons: vec![Consideration::new(
                "Distance".to_string(),
                map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
                ConsiderationParam {
                    t: ResponseCurveType::Linear,
                    m: -1.0 / 100.0,
                    k: 1.0,
                    c: 2.0,
                    b: 1.0,
                },
                ),
                Consideration::new(
                    "in sight".to_string(),
                    map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
                    ConsiderationParam {
                        t: ResponseCurveType::LessThan,
                        m: viewshed.range as f32,
                        k: 1.0,
                        c: 2.0,
                        b: 1.0,
                    },
                )],
            priority: 1.0,
        });

        potential_actions.push(Action {
            intent: Intent {
                name: "Attack enemy".to_string(),
                task: Task::Attack,
                target: vec![Target::from(*epoint)],
                turn: *turn,
            },
            cons: vec![Consideration::new(
                "Distance".to_string(),
                map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
                ConsiderationParam {
                    t: ResponseCurveType::LessThan,
                    m: 2.1,
                    k: 1.0,
                    c: 1.0,
                    b: 1.0,
                },
            )],
            priority: 2.0,
        });
    }

    potential_actions
}
