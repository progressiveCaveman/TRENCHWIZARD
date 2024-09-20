use serde::{Serialize, Deserialize};
use shipyard::{Component, AllStorages, UniqueView, View, EntityId, Get, IntoIter, IntoWithId};

use crate::{components::{Turn, self, Position, Inventory, Item, ItemType, SpatialKnowledge}, utils::Target, map::Map, tiles::TileType};

pub struct AI {}

impl AI {
    pub fn choose_intent(actions: Vec<Action>, store: &AllStorages, id: EntityId) -> Intent {
        if actions.len() < 1 {
            panic!("No actions to choose from");
        }

        let mut best = (0.0, Intent::idle());

        for i in 0..actions.len() {
            let action = &actions[i];
            let score = action.evaluate(store, id);

            // println!("Action: {}, score: {}", action.name, score);

            if score.0 > best.0 {
                best = score;
            }
        }

        best.1.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Action {
    pub cons: Vec<Consideration>,
    pub priority: f32,
    pub intent: IntentArchetype,
}

impl Action {
    pub fn evaluate(&self, store: &AllStorages, id: EntityId) -> (f32, Intent) {
        let vpos = store.borrow::<View<Position>>().unwrap();
        let vinv = store.borrow::<View<Inventory>>().unwrap();
        let vitem = store.borrow::<View<Item>>().unwrap();

        let map = store.borrow::<UniqueView<Map>>().unwrap();

        // select targets for each intent
        let intents = self.expand_intent_archetype(store, id);

        if intents.len() == 0 {
            return (0.0, Intent::idle());
        }

        let mut best = (0.0, intents[0].clone());

        for intent in intents {

            // get average of all consideration scores
            let mut scores: Vec<f32> = vec![];
            for c in self.cons.iter() {

                // calculate input from intent
                let input = match c.input_type {
                    InputType::Const => 1.0,
                    InputType::DistanceTo(_) => {
                        let pos = vpos.get(intent.owner).unwrap();
                        let mut dist = 1000000000.0;
                        for ps in pos.ps.iter() {
                            let newdist = map.distance(&vpos, Target::from(*ps), Target::from(intent.target[0]));
                            if newdist < dist {
                                dist = newdist;
                            }
                        }

                        dist
                    },
                    InputType::Inventory(target) => {
                        let inv = vinv.get(intent.owner).unwrap();
        
                        match target {
                            InputTargets::Log => {
                                inv.count_type(&vitem, ItemType::Log) as f32
                            },
                            InputTargets::Fish => {
                                inv.count_type(&vitem, ItemType::Fish) as f32
                            },
                            _ => todo!()
                        }
                    },
                };
                
                let s = c.get_score(input);
    
                // if s == 0. {
                //     return (0.0);
                // }
    
                scores.push(s);
            }

            let score = average(&scores) * self.priority;

            if score > best.0 {
                best = (score, intent.clone());
            }
        }

        best


        // multiply by priorities
        // (ave * self.priority)
    }

    fn expand_intent_archetype(&self, store: &AllStorages, id: EntityId) -> Vec<Intent> {
        let turn = store.borrow::<UniqueView<Turn>>().unwrap();
        let map = store.borrow::<UniqueView<Map>>().unwrap();
        let vinv = store.borrow::<View<Inventory>>().unwrap();
        let vitem = store.borrow::<View<Item>>().unwrap();
        let vspace = store.borrow::<View<SpatialKnowledge>>().unwrap();

        let mut intents: Vec<Intent> = vec![];

        // For each possible target to given task, evaluate scores and return best one
        match self.intent.task {
            Task::Fish => { 
                let space = vspace.get(id).unwrap();

                for target in space.get_targets(store, InputTargets::Water) {
                    match target {
                        Target::LOCATION(point) => {
                            // todo actually path to water to test if it should be considered?
                            let mut point = point;
                            point.y -= 1;
                            let aboveidx = map.point_idx(point);
                            if map.tiles[aboveidx] != TileType::Water {
                                intents.push(Intent {
                                    name: self.intent.name.clone(),
                                    owner: id,
                                    task: self.intent.task,
                                    target: vec![Target::from(point)],
                                    turn: *turn,
                                });
                            }
                        },
                        Target::ENTITY(_) => todo!(),
                    }
                }
            },
            Task::Explore => { 
                intents.push(Intent {
                    name: self.intent.name.clone(),
                    owner: id,
                    task: self.intent.task,
                    target: vec![],
                    turn: *turn,
                });
            },
            Task::Attack(target) | Task::MoveTo(target) | Task::Destroy(target) | Task::PickUpItem(target) => {
                dbg!("Looking for targets");
                if let Ok(space) = vspace.get(id) {
                    for target in space.get_targets(store, target) {
                        dbg!("Found a target");
                        intents.push(Intent {
                            name: self.intent.name.clone(),
                            owner: id,
                            task: self.intent.task,
                            target: vec![target],
                            turn: *turn,
                        });
                    }
                } else {
                    // asd
                }
                // let space = vspace.get(id).unwrap();
                // for target in space.get_targets(store, target) {
                //     dbg!("Found a target");
                //     intents.push(Intent {
                //         name: self.intent.name.clone(),
                //         owner: id,
                //         task: self.intent.task,
                //         target: vec![target],
                //         turn: *turn,
                //     });
                // }
            },
            Task::ExchangeInfo => todo!(),
            Task::DropItem => todo!(),
            Task::UseItem => todo!(),
            Task::EquipItem => todo!(),
            Task::UnequipItem => todo!(),
            Task::UseWorkshop => todo!(),
            Task::DepositItemToInventory(item_target, inv_target) => { 
                let space = vspace.get(id).unwrap();

                let inv = vinv.get(id).unwrap();
                for (itemid, item) in vitem.iter().with_id() {
                    if item_target.matches(item.typ) && inv.items.contains(&id){        
                        for inv in space.get_targets(store, inv_target) {
                            intents.push(Intent {
                                name: self.intent.name.clone(),
                                owner: id,
                                task: self.intent.task,
                                target: vec![Target::from(itemid), inv],
                                turn: *turn,
                            });
                        }
                        break;
                    }
                }
            },
            Task::Idle => { 
                intents.push(Intent {
                    name: self.intent.name.clone(),
                    owner: id,
                    task: self.intent.task,
                    target: vec![],
                    turn: *turn,
                });
            },
            Task::Spawn(_) => { 
                // todo consider neighbors
                intents.push(Intent {
                    name: self.intent.name.clone(),
                    owner: id,
                    task: self.intent.task,
                    target: vec![],
                    turn: *turn,
                });
            },
        }

        intents
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Serialize)]
pub enum Task {
    Fish,    // not an effect yet but maybe could be?
    Explore,
    ExchangeInfo,
    MoveTo(InputTargets),
    Destroy(InputTargets),
    PickUpItem(InputTargets),
    DropItem,
    UseItem,
    EquipItem,
    UnequipItem,
    UseWorkshop,
    DepositItemToInventory(InputTargets, InputTargets),
    Attack(InputTargets),
    Idle,
    Spawn(InputTargets),
}

#[derive(Component, Clone, Debug)]
pub struct Intent {
    pub name: String,
    pub owner: EntityId,
    pub task: Task,          // Tasks include input targets, which are types of things tasks care about. This is different from target below, which is intent-specific
    pub target: Vec<Target>, // most tasks have one target, more targets are specified in name, ie `DepositItemToInventory` expects [item, inventory]
    pub turn: Turn,          // turn this intent originated
}

impl Intent {
    pub fn idle() -> Self {
        Intent {
            name: "Idle".to_string(),
            owner: EntityId::default(),
            task: Task::Idle,
            target: Vec::new(),
            turn: components::Turn(0),
        }
    }
}

// Actions are stored using archetype, and specific intents are generated on the fly
#[derive(Component, Clone, Debug, PartialEq, Serialize)]
pub struct IntentArchetype {
    pub name: String,
    pub task: Task,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Consideration {
    pub name: String,
    pub input_type: InputType,
    pub params: ConsiderationParam,
}

impl Consideration {
    pub fn new(name: String, input_type: InputType, params: ConsiderationParam) -> Consideration {
        Consideration {
            name: name,
            input_type: input_type,
            params: params,
        }
    }

    fn get_score(&self, input: f32) -> f32 {
        let t = &self.params.t;
        let m = self.params.m;
        let k = self.params.k;
        let c = self.params.c;
        let b = self.params.b;

        let score = match t {
            ResponseCurveType::Const => m * input,
            ResponseCurveType::Quadratic | ResponseCurveType::Linear => m * (input - c).powf(k) + b,
            ResponseCurveType::Logistic => {
                let e = std::f64::consts::E as f32;
                k * 1. / (1. + (1000. * e * m).powf(-1. * input + c)) + b
            }
            ResponseCurveType::GreaterThan => {
                if input > m {
                    1.
                } else {
                    0.
                }
            }
            ResponseCurveType::LessThan => {
                if input < m {
                    1.
                } else {
                    0.
                }
            }
        };

        return score.clamp(0., 1.);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsiderationParam {
    pub t: ResponseCurveType,
    pub m: f32,
    pub k: f32,
    pub c: f32,
    pub b: f32,
}

impl ConsiderationParam {
    pub fn new_const(v: f32) -> ConsiderationParam {
        ConsiderationParam {
            t: ResponseCurveType::Const,
            m: v,
            k: 0.,
            c: 0.,
            b: 0.,
        }
    }
}

/*
for types Const, GreaterThan, and LessThan, only m is considered
Linear
Quadratic
logisitic
Logit

Paramters - m,k,c,b

Linear/quad: y=m*(x-c)^k + b
m = slope
k = exponent
b = vert shift
c = horiz shift

Logistic: y = (k * (1/(1+1000em^(-1x+c)))) + b
m=slope of inflection
k=vertical size of curve
b=vert shift
c=horiz shift
*/
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResponseCurveType {
    Const,
    GreaterThan,
    LessThan,
    Linear,
    Quadratic,
    Logistic,
}

pub fn average(numbers: &[f32]) -> f32 {
    let sum: f32 = numbers.iter().sum();
    let count = numbers.len() as f32;
    sum / count
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    Const, // used as a baseline for things
    DistanceTo(InputTargets),
    Inventory(InputTargets), // intent owner's inventory
    // TargetInventory(InputTargets), // target's inventory
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputTargets {
    Tree,
    Log,
    LumberMill,
    Water,
    Fishery,
    Enemy,
    Fish,
    Player,
    None,
    Orc,
}

impl InputTargets {
    pub fn matches(&self, item: ItemType) -> bool {
        match self {
            InputTargets::Log => item == ItemType::Log,
            InputTargets::Fish => item == ItemType::Fish,
            _ => false,
        }
    }
}