use crate::ai::decisions::{Intent, Task};
use crate::components::{Position, Tree};
use crate::entity_factory;
use crate::utils::{InvalidPoint, Target};
use rltk::Point;
use shipyard::{AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, View};

pub fn run_dissasemble_system(mut all_storages: AllStoragesViewMut) {
    let mut to_spawn_log: Vec<(i32, i32)> = vec![];
    let mut to_delete: Vec<EntityId> = vec![];

    {
        let vpos = all_storages.borrow::<View<Position>>().unwrap();
        let vintent = all_storages.borrow::<View<Intent>>().unwrap();
        let vtree = all_storages.borrow::<View<Tree>>().unwrap();

        for (_, (pos, intent)) in (&vpos, &vintent).iter().with_id() {
            if let Task::Destroy(_) = intent.task {
                let target = intent.target[0].get_point(&vpos);

                if target == Point::invalid_point() {
                    continue;
                }

                // check distance
                for p in pos.ps.iter() {
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(target, *p);
                    if distance > 1.5 {
                        // dbg!("entity not next to target", distance);
                        continue;
                    }

                    if let Target::ENTITY(e) = intent.target[0] {
                        let mut spawn_log = false;
                        if let Ok(_) = vtree.get(e) {
                            spawn_log = true;
                        }

                        let tpoint = if let Ok(p) = vpos.get(e) {
                            p.ps[0]
                        } else {
                            dbg!("No position");
                            Point::invalid_point()
                        };

                        if spawn_log {
                            // entity_factory::log(&mut all_storages, tpoint.x, tpoint.y);
                            to_spawn_log.push((tpoint.x, tpoint.y));
                        }

                        // all_storages.delete_entity(e);
                        to_delete.push(e);
                    }

                    break;
                }
            }
        }
    }

    for (x, y) in to_spawn_log {
        entity_factory::log(&mut all_storages, (x, y));
    }

    for e in to_delete {
        all_storages.delete_entity(e);
    }
}

// pub fn run_dissasemble_system(mut all_storages: AllStoragesViewMut, vpos: View<Position>, vintent: View<Intent>, vtree: View<Tree>) {
