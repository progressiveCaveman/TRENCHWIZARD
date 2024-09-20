use crate::ai::decisions::Action;
use crate::components::{Aging, Actor, ActorType, Turn, RNG, PlankHouse, Position};
use crate::effects::{add_effect, EffectType, Targets};
use crate::entity_factory::EntitySpawnTypes;
use crate::map::Map;
use shipyard::{IntoIter, IntoWithId, UniqueViewMut, View, ViewMut, EntityId, Get};

pub fn run_time_system(
    map: UniqueViewMut<Map>,
    mut turn: UniqueViewMut<Turn>,
    mut rng: UniqueViewMut<RNG>,
    vactor: View<Actor>,
    mut vaging: ViewMut<Aging>,
    vhouse: ViewMut<PlankHouse>,
    vpos: ViewMut<Position>,
) {
    turn.0 += 1;

    let maxage = 250;

    for (id, (aging, actor)) in (&mut vaging, &vactor).iter().with_id() {
        aging.turns += 1;

        if actor.atype == ActorType::Villager {
            // let newactions = randomize_actions(actor, &mut rng);
            if aging.turns > maxage && rng.0.roll_dice(1, 3) < 2{
                add_effect(None, EffectType::Delete { entity: id });

                let mut houses = vec![];
                for (_, (_, pos)) in (&vhouse, &vpos).iter().with_id() {
                    let mut point = pos.ps[0];
                    point.y -= 1;
                    houses.push(map.point_idx(point));
                }

                let partner = pick_top_villager(&vactor, &mut rng);
                let partner_actor = vactor.get(partner).unwrap();
                let mut actions = merge_actions(actor.actions.clone(), partner_actor.actions.clone(), 0.3, &mut rng);
        
                let idx = rng.0.roll_dice(1, houses.len() as i32) as usize - 1;
                add_effect(None, EffectType::Spawn { 
                    etype: EntitySpawnTypes::Villager, 
                    target: Targets::Tile { tile_idx: houses[idx] }, 
                    actions: Some(randomize_actions(&mut actions, &mut rng)) 
                });
            } else {
            }
        }
    }
}

pub fn randomize_actions(actions: &mut Vec<Action>, rng: &mut UniqueViewMut<RNG>) -> Vec<Action> {
    let random_factor = 0.01;

    for action in actions.iter_mut() {
        for con in action.cons.iter_mut() {
            let amt = con.params.m * random_factor;
            con.params.m = con.params.m - amt + 2. * (rng).0.rand::<f32>() * amt;

            let amt = con.params.k * random_factor;
            con.params.k = con.params.k - amt + 2. * rng.0.rand::<f32>() * amt;

            let amt = con.params.b * random_factor;
            con.params.b = con.params.b - amt + 2. * rng.0.rand::<f32>() * amt;

            let amt = con.params.c * random_factor;
            con.params.c = con.params.c - amt + 2. * rng.0.rand::<f32>() * amt;
        }
    }

    actions.to_vec()
}

// bias is a value between [0, 1] representing which action set should be favored
pub fn merge_actions(a1: Vec<Action>, a2: Vec<Action>, bias: f32, rng: &mut UniqueViewMut<RNG>) -> Vec<Action> {
    let mut new_actions = a1.clone();
 
    for (aidx, action) in a2.iter().enumerate() {
        for (cidx, con) in action.cons.iter().enumerate() {
            if rng.0.rand::<f32>() > bias {
                new_actions[aidx].cons[cidx] = con.clone(); 
            }
        }   
    }   

    new_actions
}

// randomly selects one of the top scoring villagers
pub fn pick_top_villager(vactor: &View<Actor>, rng: &mut UniqueViewMut<RNG>) -> EntityId {
    let mut top3 = vec![];

    // get the top 3 villagers
    for (id, actor) in (&vactor).iter().with_id() {
        if actor.atype != ActorType::Villager {
            continue;
        }

        if top3.len() < 3 {
            top3.push((id, actor.score));
        } else {
            for i in 0..top3.len() {
                if actor.score > top3[i].1 {
                    top3.insert(i, (id, actor.score));
                    top3.pop();
                    break;
                }
            }
        }
    }

    if top3.is_empty() {
        dbg!("ERROR: No villager found");
    }

    let ridx = rng.0.roll_dice(1, 3) - 1;
    top3[ridx as usize].0
}

pub fn breed_villagers(vactor: View<Actor>, host: EntityId, ) -> Vec<Action> {
    let mut top3 = vec![];

    // let score = if let Ok(actor) = vactor.get(host) {
    //     actor.score
    // } else {
    //     0
    // };

    // get the top 3 villagers
    for (id, actor) in (&vactor).iter().with_id() {
        if id == host {
            continue;
        }

        if top3.len() < 3 {
            top3.push((id, actor.score));
        } else {
            for i in 0..top3.len() {
                if actor.score > top3[i].1 {
                    top3.insert(i, (id, actor.score));
                    top3.pop();
                    break;
                }
            }
        }
    }

     vec![]
}