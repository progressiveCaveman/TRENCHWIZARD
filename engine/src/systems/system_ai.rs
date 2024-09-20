use crate::ai::decisions::{Intent, Task, InputTargets, AI};
use crate::ai::labors::{get_actions, AIBehaviors};
use crate::components::{Actor, ActorType, DijkstraMapToMe, Faction, Position, Spawner, SpawnerType, Turn, PlayerID, Vision, Item, ItemType};
use crate::effects::{add_effect, EffectType};
use crate::entity_factory::{self, EntitySpawnTypes};
use crate::map::Map;
use crate::tiles::TileType;
use crate::utils::vision::vision_contains;
use crate::utils::{get_neighbors, Target, InvalidPoint};
use rltk::{BaseMap, Point};
use shipyard::{AddComponent, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, UniqueView, View, ViewMut, UniqueViewMut};

pub fn run_ai_system(mut store: AllStoragesViewMut) {
    let mut to_move_from_to: Vec<(EntityId, Point, Point)> = vec![];
    let mut to_fish: Vec<(EntityId, Point)> = vec![];
    let mut to_attack: Vec<(EntityId, Point)> = vec![];
    let mut to_spawn_fish: Vec<Point> = vec![];
    let mut to_spawn_orc: Vec<(Point, Faction)> = vec![];
    let mut to_deposit_items: Vec<(EntityId, Intent)> = vec![];

    store.run(
        |map: UniqueView<Map>,
         turn: UniqueView<Turn>,
         playerid: UniqueView<PlayerID>,
         vactor: View<Actor>,
         vpos: View<Position>,
         vvision: View<Vision>,
         vdijkstra: View<DijkstraMapToMe>,
         mut vintent: ViewMut<Intent>,
         vspawner: ViewMut<Spawner>| {
            for (id, (actor, pos)) in (&vactor, &vpos).iter().with_id() {
                let new_intent = match actor.atype {
                    ActorType::Player => continue,
                    ActorType::Fish => continue,
                    ActorType::Orc | ActorType::Wolf=> {

                        dbg!(&actor.actions);
                        //TODO temp change for orc arena
                        AI::choose_intent(actor.actions.clone(), &store, id) //todo clone here is messy

                        // if let Ok(vision) = vvision.get(id) {
                        //     if vision_contains(&store, vision.clone(), playerid.0) { //todo vision.clone bad
                        //         Intent {
                        //             name: "Attack player".to_string(),
                        //             owner: id,
                        //             task: Task::Attack(InputTargets::Player),
                        //             target: vec![Target::ENTITY(playerid.0)],
                        //             turn: *turn,
                        //         }
                        //     } else {
                        //         continue;
                        //     }
                        // } else {
                        //     continue;
                        // }
                    },
                    ActorType::Villager => AI::choose_intent(actor.actions.clone(), &store, id), //todo clone here is messy
                    ActorType::Spawner => {


                        // add_effect(None, EffectType::Spawn { 
                        //     etype: EntitySpawnTypes::Villager, 
                        //     target: Targets::Tile { tile_idx: houses[idx] }, 
                        //     actions: Some(randomize_actions(&mut actions, &mut rng)) 
                        // });

                        if let Ok(spawner) = vspawner.get(id) {
                            if turn.0 % spawner.rate == 0 {
                                Intent {
                                    name: "spawn".to_string(),
                                    owner: id,
                                    task: Task::Spawn(match spawner.typ {
                                        SpawnerType::Orc => InputTargets::Orc,
                                        SpawnerType::Fish => InputTargets::Fish,
                                    }),
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
                                    owner: id,
                                    task: Task::Idle,
                                    target: Vec::new(),
                                    turn: *turn,
                                }
                            }
                        } else {
                            Intent {
                                name: "none".to_string(),
                                owner: id,
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
                    Task::MoveTo(_) => {
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
                    Task::Destroy(_) => {
                        // todo!() this should happen here and not in disassemble
                    }
                    Task::PickUpItem(_) => {            
                        if let Target::ENTITY(e) = new_intent.target[0] {
                            add_effect(Some(id), EffectType::PickUp { entity: e });
                        }
                    },
                    Task::DropItem => todo!(),
                    Task::UseItem => todo!(),
                    Task::EquipItem => todo!(),
                    Task::UnequipItem => todo!(),
                    Task::UseWorkshop => todo!(),
                    Task::DepositItemToInventory(..) => {
                        dbg!("asdasd");
                        to_deposit_items.push((id, new_intent));
                    }
                    Task::Attack(_) => {
                        match new_intent.target[0] {
                            Target::LOCATION(loc) => {
                                to_attack.push((id, loc));
                                
                            },
                            Target::ENTITY(target) => {
                                if let Ok(target_pos) = vpos.get(target) {
                                    to_attack.push((id, target_pos.ps[0]));
                                }
                            },
                        }
                    }
                    Task::Idle => {}
                    Task::Spawn(target) => {
                        match target {
                            InputTargets::Orc => {
                                to_spawn_orc.push((pos.ps[0], actor.faction));
                            }
                            InputTargets::Fish => {
                                to_spawn_fish.push(pos.ps[0]);
                                // entity_factory::fish(&mut store, pos.ps[0].x, pos.ps[0].y);
                            }
                            _ => {}
                        }
                    }
                }
            }
        },
    );

    for (e, from, to) in to_move_from_to {
        let map = store.borrow::<UniqueViewMut<Map>>().unwrap();

        if map.get_pathing_distance(from.to_index(map.size.0), to.to_index(map.size.0)) <= 2.1 {
            add_effect(
                Some(e),
                EffectType::Move {
                    tile_idx: to.to_index(map.size.0),
                },
            );
            continue;
        }

        // let mut pathing_grid: PathingGrid = PathingGrid::new(map.size.0, map.size.1, false);

        // pathing_grid.set(1, 1, true);
        // pathing_grid.generate_components();
        // println!("{}", pathing_grid);
        // let start = Point::new(0, 0);
        // let end = Point::new(2, 2);
        // let path = pathing_grid
        //     .get_path_single_goal(start, end, false)
        //     .unwrap();
        // println!("Path:");
        // for p in path {
        //     println!("{:?}", p);
        // }

        let path = map.get_path(from, to);

        if path.success && path.steps.len() > 1 {
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
        entity_factory::fish(&mut store, pos.to_xy());
    }

    for (pos, faction) in to_spawn_orc.iter() {
        let e = entity_factory::orc(&mut store, pos.to_xy(), &get_actions(&vec![AIBehaviors::AttackEnemies]));
        store.run(|mut vactor: ViewMut<Actor>| {
            if let Ok(spawned_actor) = (&mut vactor).get(e) {
                spawned_actor.faction = *faction;
            }
        });
    }

    for (id, _) in to_deposit_items.iter() {
        store.run(|mut vactor: ViewMut<Actor>, vintent: View<Intent>, vitem: ViewMut<Item>| {
            if let Ok((actor, intent)) = (&mut vactor, &vintent).get(*id) {
                if let Target::ENTITY(item) = intent.target[0] {
                    if let Target::ENTITY(target) = intent.target[1] {

                        dbg!("deposit items");
                        // TODO this looks like a race condition
                        add_effect(Some(*id), EffectType::Drop { entity: item });
                        add_effect(Some(target), EffectType::PickUp { entity: item });

                        // can this be exploited by the ai? 
                        // for b in actor.behaviors.iter() {
                        //     if let Ok(item) = vitem.get(item) {
                        //         if *b == AIBehaviors::GatherFish && item.typ == ItemType::Fish {
                        //             actor.score += 10;
                        //         }
                        //         if *b == AIBehaviors::GatherWood && item.typ == ItemType::Log {
                        //             actor.score += 10;
                        //         }
                        //     }
                        // }
                    }
                }
            }
        });
    }
}
