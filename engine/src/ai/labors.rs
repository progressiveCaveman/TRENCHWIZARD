use super::decisions::{Action, Consideration, ConsiderationParam, ResponseCurveType, Task, InputType, InputTargets, IntentArchetype};

#[derive(Clone, Debug, PartialEq)]
pub enum AIBehaviors {
    GatherWood,
    GatherFish,
    AttackEnemies,
    Confused,
    Wander,
}

pub fn get_actions(behaviors: &Vec<AIBehaviors>) -> Vec<Action> {

    let mut potential_actions: Vec<Action> = vec![];

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "idle".to_string(),
            task: Task::Idle,
        },
        cons: vec![Consideration::new(
            "baseline".to_string(),
            InputType::Const,
            ConsiderationParam::new_const(0.1),
        )],
        priority: 1.0,
    });

    for b in behaviors.iter() {
        match b {
            AIBehaviors::GatherWood => potential_actions.append(&mut get_gather_wood_actions()),
            AIBehaviors::GatherFish => potential_actions.append(&mut get_gather_fish_actions()),
            AIBehaviors::AttackEnemies => potential_actions.append(&mut get_attack_actions()),
            _ => {} // AIBehaviors::Wander => ,
        }
    }

    potential_actions
    // return AI::choose_intent(potential_actions, store, id);
}

pub fn get_gather_wood_actions() -> Vec<Action> {
    let mut potential_actions: Vec<Action> = vec![];

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "go to tree".to_string(),
            task: Task::MoveTo(InputTargets::Tree),
        },
        cons: vec![
            Consideration::new(
                "Distance".to_string(),
                InputType::DistanceTo(InputTargets::Tree),
                // map.distance(&vpos, Target::from(pos), Target::from(tree)),
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
        intent: IntentArchetype {
            name: "chop tree".to_string(),
            task: Task::Destroy(InputTargets::Tree),
        },
        cons: vec![
            Consideration::new(
                "Distance to tree".to_string(),
                InputType::DistanceTo(InputTargets::Tree),
                // map.distance(&vpos, Target::from(pos), Target::from(tree)),
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

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "pick up wood".to_string(),
            task: Task::PickUpItem(InputTargets::Log),
        },
        cons: vec![
            Consideration::new(
                "Distance to log".to_string(),
                InputType::DistanceTo(InputTargets::Log),
                // map.distance(&vpos, Target::from(pos), Target::from(*log)),
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

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "move to lumber mill".to_string(),
            task: Task::MoveTo(InputTargets::LumberMill),
        },
        cons: vec![
            Consideration::new(
                "Distance to lm".to_string(),
                InputType::DistanceTo(InputTargets::LumberMill),
                // map.distance(&vpos, Target::from(pos), Target::from(lm)),
                ConsiderationParam {
                    t: ResponseCurveType::Linear,
                    m: -1. / 20.,
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
                "logs in inv".to_string(),
                InputType::Inventory(InputTargets::Log),
                ConsiderationParam {
                    t: ResponseCurveType::GreaterThan,
                    m: 5.0,
                    k: 1.0,
                    c: 0.0,
                    b: 0.0,
                },
            ),
            Consideration::new(
                "logs in iventory".to_string(),
                InputType::Inventory(InputTargets::Log),
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

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "deposit logs at lumber mill".to_string(),
            task: Task::DepositItemToInventory(InputTargets::Log, InputTargets::LumberMill),
        },
        cons: vec![
            Consideration::new(
                "Distance to lm".to_string(),
                InputType::DistanceTo(InputTargets::LumberMill),
                // map.distance(&vpos, Target::from(pos), Target::from(lm)),
                ConsiderationParam {
                    t: ResponseCurveType::LessThan,
                    m: 2.1,
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
                InputType::Inventory(InputTargets::Log),
                ConsiderationParam {
                    t: ResponseCurveType::Linear,
                    m: 1. / 5.0,
                    k: 1.0,
                    c: 0.0,
                    b: 0.0,
                },
            ),
        ],
        priority: 3.0,
    });

    // wander action
    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "explore".to_string(),
            task: Task::Explore,
        },
        cons: vec![Consideration::new(
            "baseline".to_string(),
            InputType::Const,
            ConsiderationParam::new_const(0.1),
        )],
        priority: 1.0,
    });

    potential_actions
}

pub fn get_gather_fish_actions() -> Vec<Action> {
    let mut potential_actions: Vec<Action> = vec![];

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "go to water".to_string(),
            task: Task::MoveTo(InputTargets::Water),
        },
        cons: vec![
            Consideration::new(
                "Distance".to_string(),
                InputType::DistanceTo(InputTargets::Water),
                // map.distance(&vpos, Target::from(pos), Target::from(wp)),
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
        intent: IntentArchetype {
            name: "fish at water".to_string(),
            task: Task::Fish,
        },
        cons: vec![
            Consideration::new(
                "Distance".to_string(),
                InputType::DistanceTo(InputTargets::Water),
                // map.distance(&vpos, Target::from(pos), Target::from(wp)),
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

    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "move to fishery".to_string(),
            task: Task::MoveTo(InputTargets::Fishery),
        },
        cons: vec![
            Consideration::new(
                "Distance".to_string(),
                InputType::DistanceTo(InputTargets::Fishery),
                // map.distance(&vpos, Target::from(pos), Target::from(f)),
                ConsiderationParam {
                    t: ResponseCurveType::Linear,
                    m: 1. - 1. / 20.,
                    k: 1.0,
                    c: 1.0,
                    b: 0.0,
                },
            ),
            // Consideration::new(
            //     "fish in stockpile".to_string(),
            //     finv_count,
            //     ConsiderationParam {
            //         t: ResponseCurveType::Linear,
            //         m: -1. / 50.0,
            //         k: 1.0,
            //         c: 0.0,
            //         b: 1.0,
            //     },
            // ),
            Consideration::new(
                "fish in iventory".to_string(),
                InputType::Inventory(InputTargets::Fish),
                // fish_in_inv as f32,
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
        intent: IntentArchetype {
            name: "deposit fish at fishery".to_string(),
            task: Task::DepositItemToInventory(InputTargets::Fish, InputTargets::Fishery),
        },
        cons: vec![
            Consideration::new(
                "Distance to fishery".to_string(),
                InputType::DistanceTo(InputTargets::Fishery),
                // map.distance(&vpos, Target::from(pos), Target::from(f)),
                ConsiderationParam {
                    t: ResponseCurveType::LessThan,
                    m: 2.,
                    k: 2.0,
                    c: 1.0,
                    b: 0.0,
                },
            ),
            // Consideration::new(
            //     "fish in stockpile".to_string(),
            //     finv_count,
            //     ConsiderationParam {
            //         t: ResponseCurveType::Linear,
            //         m: -1. / 50.0,
            //         k: 1.0,
            //         c: 0.0,
            //         b: 1.0,
            //     },
            // ),
            Consideration::new(
                "fish in iventory".to_string(),
                InputType::Inventory(InputTargets::Fish),
                // fish_in_inv as f32,
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

    potential_actions
}

pub fn get_attack_actions() -> Vec<Action> {
    let mut potential_actions: Vec<Action> = vec![];
    potential_actions.push(Action {
        intent: IntentArchetype {
            name: "go to enemy".to_string(),
            task: Task::MoveTo(InputTargets::Enemy),
        },
        cons: vec![
            Consideration::new(
                "Distance".to_string(),
                InputType::DistanceTo(InputTargets::Enemy),
                // map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
                ConsiderationParam {
                    t: ResponseCurveType::Linear,
                    m: -1.0 / 100.0,
                    k: 1.0,
                    c: 10.0,
                    b: 1.0,
                },
            // ),
            // Consideration::new(
            //     "in sight".to_string(),
            //     InputType::DistanceTo(InputTargets::Enemy),
            //     // map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
            //     ConsiderationParam {
            //         t: ResponseCurveType::LessThan,
            //         m: viewshed.range as f32,
            //         k: 1.0,
            //         c: 2.0,
            //         b: 1.0,
            //     },
            )],
        priority: 1.0,
    });

    potential_actions.push(Action {
        intent: IntentArchetype { 
            name: "Attack enemy".to_string(),
            task: Task::Attack(InputTargets::Enemy),
        },
        cons: vec![Consideration::new(
            "Distance".to_string(),
            InputType::DistanceTo(InputTargets::Enemy),
            // map.distance(&vpos, Target::from(pos), Target::from(*epoint)),
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

    potential_actions
}
