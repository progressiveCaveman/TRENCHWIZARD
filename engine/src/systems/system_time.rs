use crate::components::{Aging, Actor, ActorType, Turn, RNG};
use crate::effects::{add_effect, EffectType};
use shipyard::{IntoIter, IntoWithId, UniqueViewMut, View, ViewMut};

pub fn run_time_system(
    mut turn: UniqueViewMut<Turn>,
    mut rng: UniqueViewMut<RNG>,
    vactor: View<Actor>,
    mut vaging: ViewMut<Aging>,
) {  
    turn.0 += 1;

    

    for (id, (aging, actor)) in (&mut vaging, &vactor).iter().with_id() {
        aging.turns += 1;

        if actor.atype == ActorType::Villager {
            if aging.turns > 50 && rng.0.roll_dice(1, 3) < 2{
                add_effect(None, EffectType::Delete { entity: id });
            }
        }
    }
}
