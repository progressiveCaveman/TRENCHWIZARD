use crate::world::components::{Actor, ActorType, Position};
use crate::world::effects::{add_effect, EffectType};
use crate::world::map::Map;
use crate::tiles::TileType;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rltk::Point;
use shipyard::{EntityId, IntoIter, IntoWithId, UniqueView, View};

// Leaving this in a separate system for now as I suspect AI is going to change significantly

// currently fish only move east
pub fn run_fish_ai(map: UniqueView<Map>, vpos: View<Position>, vactor: View<Actor>) {
    let mut to_try_move: Vec<(EntityId, Point)> = vec![];
    let mut to_remove: Vec<EntityId> = vec![];

    for (id, (pos, actor)) in (&vpos, &vactor).iter().with_id() {
        if actor.atype != ActorType::Fish {
            continue;
        }

        if pos.ps.len() == 1 {
            // if at edge of map, remove fish

            let pos = pos.ps[0];

            if pos.x >= map.size.0 - 2 {
                to_remove.push(id);
            } else {
                to_try_move.push((id, pos));
            }
        } else {
            dbg!("ERROR: multi-tile fish not supported");
        }
    }

    for (e, pos) in to_try_move {
        let mut potential_spaces = vec![
            Point { x: pos.x + 1, y: pos.y },
            Point {
                x: pos.x + 1,
                y: pos.y + 1,
            },
            Point {
                x: pos.x + 1,
                y: pos.y - 1,
            },
        ];

        potential_spaces.shuffle(&mut thread_rng());

        for ps in potential_spaces {
            let canmove = {
                let idx = map.point_idx(ps);
                map.tiles[idx] == TileType::Water
            };

            if canmove {
                add_effect(
                    Some(e),
                    EffectType::Move {
                        tile_idx: map.point_idx(ps),
                    },
                );
                // movement::try_move_entity(e, point_diff(pos, ps), gs);
                break;
            }
        }
    }

    for e in to_remove.iter() {
        add_effect(Some(*e), EffectType::Delete { entity: *e });
        // gs.world.delete_entity(*e);
        // dbg!("ERROR: Need to delete entity");
    }
}
