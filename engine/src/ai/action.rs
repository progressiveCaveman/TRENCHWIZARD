use serde::Serialize;
use shipyard::{AllStorages, UniqueView, View, EntityId, Get, IntoIter, IntoWithId};

use crate::{simulator::{components::{Inventory, Item, ItemType, Position, SpatialKnowledge, Turn}, map::Map}, tiles::TileType, utils::Target};
use crate::ai::intent::IntentArchetype;
use super::{consideration::Consideration, input::{InputTargets, InputType}, intent::{Intent, Task}};


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
                let space = vspace.get(id).unwrap();
                for target in space.get_targets(store, target) {
                    intents.push(Intent {
                        name: self.intent.name.clone(),
                        owner: id,
                        task: self.intent.task,
                        target: vec![target],
                        turn: *turn,
                    });
                }
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

pub fn average(numbers: &[f32]) -> f32 {
    let sum: f32 = numbers.iter().sum();
    let count = numbers.len() as f32;
    sum / count
}
