use crate::ai::decisions::{Intent, Target, Task};
use crate::ai::labors;
use crate::components::{Actor, ActorType, DijkstraMapToMe, Faction, Position, Spawner, SpawnerType};
use crate::effects::{add_effect, EffectType};
use crate::entity_factory;
use crate::map::{Map, TileType};
use crate::uniques::Turn;
use crate::utils::{get_neighbors, get_path};
use rltk;
use rltk::{BaseMap, Point};
use shipyard::{AddComponent, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, UniqueView, View, ViewMut};

pub fn run_ai_system(mut store: AllStoragesViewMut) {
    let mut to_move_from_to: Vec<(EntityId, Point, Point)> = vec![];
    let mut to_fish: Vec<(EntityId, Point)> = vec![];
    let mut to_attack: Vec<(EntityId, Point)> = vec![];
    let mut to_spawn_fish: Vec<Point> = vec![];
    let mut to_spawn_orc: Vec<(Point, Faction)> = vec![];

    store.run(
        |map: UniqueView<Map>,
         turn: UniqueView<Turn>,
         vactor: View<Actor>,
         vpos: View<Position>,
         vdijkstra: View<DijkstraMapToMe>,
         mut vintent: ViewMut<Intent>,
         vspawner: ViewMut<Spawner>| {
            for (id, (actor, pos)) in (&vactor, &vpos).iter().with_id() {
                // if actor.atype != ActorType::Villager && actor.atype != ActorType::Orc {
                //     continue;
                // }

                let new_intent = match actor.atype {
                    ActorType::Player => continue,
                    ActorType::Orc | ActorType::Villager | ActorType::Wolf => labors::get_action(&store, id).intent,
                    ActorType::Fish => continue,
                    ActorType::Spawner => {
                        if let Ok(spawner) = vspawner.get(id) {
                            if turn.0 % spawner.rate == 0 {
                                Intent {
                                    name: "spawn".to_string(),
                                    task: Task::Spawn,
                                    target: Vec::new(),
                                    turn: *turn,
                                }
                                // to_spawn.push((
                                //     Point {
                                //         x: fpos.x,
                                //         y: fpos.y + 1,
                                //     },
                                //     actor.faction,
                                //     spawner.typ,
                                // ));
                            } else {
                                Intent {
                                    name: "none".to_string(),
                                    task: Task::Idle,
                                    target: Vec::new(),
                                    turn: *turn,
                                }
                            }
                        } else {
                            Intent {
                                name: "none".to_string(),
                                task: Task::Idle,
                                target: Vec::new(),
                                turn: *turn,
                            }
                        }
                    }
                };

                // let new_intent = labors::get_action(&store, id).intent;
                vintent.add_component_unchecked(id, new_intent.clone());

                //world.query::<(&Villager, &mut Position, &mut Intent)>().iter() {
                match new_intent.task {
                    Task::Fish => {
                        to_fish.push((id, pos.ps[0]));
                    }
                    Task::Explore => add_effect(Some(id), EffectType::Explore {}),
                    Task::ExchangeInfo => todo!(),
                    Task::MoveTo => {
                        if let Target::ENTITY(target) = new_intent.target[0] {
                            if let Ok(target_pos) = vpos.get(target) {
                                //world.get::<Position>(target) {
                                if let Ok(dijkstra) = vdijkstra.get(target) {
                                    //world.get::<DijkstraMapToMe>(target) {
                                    let my_idx = map.point_idx(pos.ps[0]);
                                    let neighbor_indices = map.get_available_exits(my_idx);

                                    let mut tidx: i32 = -1;
                                    for &i in neighbor_indices.iter() {
                                        if tidx == -1 || dijkstra.map.map[i.0] < dijkstra.map.map[tidx as usize] {
                                            tidx = i.0 as i32;
                                        }
                                    }

                                    to_move_from_to.push((id, pos.ps[0], map.idx_point(tidx as usize)));
                                } else {
                                    to_move_from_to.push((id, pos.ps[0], target_pos.ps[0]));
                                }
                            }
                        } else if let Target::LOCATION(loc) = new_intent.target[0] {
                            to_move_from_to.push((id, pos.ps[0], loc));
                        }
                    }
                    Task::Destroy => {}
                    Task::PickUpItem => {}
                    Task::DropItem => todo!(),
                    Task::UseItem => todo!(),
                    Task::EquipItem => todo!(),
                    Task::UnequipItem => todo!(),
                    Task::UseWorkshop => todo!(),
                    Task::DepositItemToInventory => {}
                    Task::Attack => {
                        if let Target::ENTITY(target) = new_intent.target[0] {
                            if let Ok(target_pos) = vpos.get(target) {
                                dbg!(1);
                                to_attack.push((id, target_pos.ps[0]));
                            }
                        } else if let Target::LOCATION(loc) = new_intent.target[0] {
                            to_attack.push((id, loc));
                        }
                    }
                    Task::Idle => {}
                    Task::Spawn => {
                        if let Ok(spawner) = vspawner.get(id) {
                            match spawner.typ {
                                SpawnerType::Orc => {
                                    to_spawn_orc.push((pos.ps[0], actor.faction));
                                }
                                SpawnerType::Fish => {
                                    to_spawn_fish.push(pos.ps[0]);
                                    // entity_factory::fish(&mut store, pos.ps[0].x, pos.ps[0].y);
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    for (e, from, to) in to_move_from_to {
        let map = store.borrow::<UniqueView<Map>>().unwrap();
        let path = get_path(&map, from, to);

        if path.success && path.steps.len() > 1 {
            // movement::try_move_entity(e, point_diff(from, p), gs);
            add_effect(
                Some(e),
                EffectType::Move {
                    tile_idx: path.steps[1],
                },
            );
        }
    }

    for (e, p) in to_fish {
        let map = store.borrow::<UniqueView<Map>>().unwrap();

        let n = get_neighbors(p);
        let adj_water: Vec<&Point> = n
            .iter()
            .filter(|p| {
                let idx = map.point_idx(**p);
                map.tiles[idx] == TileType::Water
            })
            .collect();

        for p in adj_water.iter() {
            let idx = map.point_idx(**p);
            for te in &map.tile_content[idx] {
                let vactor = store.borrow::<View<Actor>>().unwrap();
                if let Ok(actor) = vactor.get(*te) {
                    if actor.atype == ActorType::Fish {
                        //found a target
                        add_effect(Some(e), EffectType::PickUp { entity: *te });
                        break;
                    }
                }
            }
        }
    }

    for (eid, point) in to_attack.iter() {
        let map = store.borrow::<UniqueView<Map>>().unwrap();

        add_effect(
            Some(*eid),
            EffectType::MoveOrAttack {
                tile_idx: map.point_idx(*point),
            },
        );
    }

    for pos in to_spawn_fish.iter() {
        entity_factory::fish(&mut store, pos.x, pos.y);
    }

    for (pos, faction) in to_spawn_orc.iter() {
        let e = entity_factory::orc(&mut store, pos.x, pos.y);
        store.run(|mut vactor: ViewMut<Actor>| {
            if let Ok(mut spawned_actor) = (&mut vactor).get(e) {
                spawned_actor.faction = *faction;
            } else {
                dbg!("Error: Orc isn't an actor, this shouldn't happen");
            }
        });
    }
}
