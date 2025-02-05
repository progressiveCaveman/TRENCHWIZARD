use crate::simulator::components::{Aging, Actor, ActorType, Turn, RNG, PlankHouse, Position};
use crate::simulator::effects::{add_effect, EffectType, Targets};
use crate::entity_factory::EntitySpawnTypes;
use crate::simulator::map::Map;
use shipyard::{IntoIter, IntoWithId, UniqueViewMut, View, ViewMut};

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

    let mut num_villagers = 0;

    for (id, (aging, actor)) in (&mut vaging, &vactor).iter().with_id() {
        aging.turns += 1;

        if actor.atype == ActorType::Villager {
            if aging.turns > 250 && rng.0.roll_dice(1, 3) < 2{
                add_effect(None, EffectType::Delete { entity: id });
            } else {
                num_villagers += 1;
            }
        }
    }

    if num_villagers > 0 && num_villagers < 20 {
        let mut houses = vec![];
        for (_, (_, pos)) in (&vhouse, &vpos).iter().with_id() {
            let mut point = pos.ps[0];
            point.y -= 1;
            houses.push(map.point_idx(point));
        }

        let idx = rng.0.roll_dice(1, houses.len() as i32) as usize - 1;
        add_effect(None, EffectType::Spawn { etype: EntitySpawnTypes::Villager, target: Targets::Tile { tile_idx: houses[idx] } })
    }
}
