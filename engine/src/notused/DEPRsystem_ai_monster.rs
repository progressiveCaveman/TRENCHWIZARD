use crate::components::{Confusion, Position, Viewshed, WantsToAttack, Actor, ActorType};
use crate::effects::{add_effect, EffectType};
use crate::map::Map;
use crate::palette::Palette;
use crate::systems::system_particle::ParticleBuilder;
use crate::utils::{self, PPoint, PlayerID};
use rltk;
use shipyard::{
    AddComponent, AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueView,
    UniqueViewMut, View, ViewMut,
};

pub fn run_monster_ai_system(store: AllStoragesViewMut) {
    let mut map = store.borrow::<UniqueViewMut<Map>>().unwrap();
    let player_id = store.borrow::<UniqueView<PlayerID>>().unwrap().0;
    let ppos = store.borrow::<UniqueView<PPoint>>().unwrap().0;
    let mut particle_builder = store.borrow::<UniqueViewMut<ParticleBuilder>>().unwrap();

    let vpos = store.borrow::<View<Position>>().unwrap();
    let vvs = store.borrow::<View<Viewshed>>().unwrap();
    let mut vconfusion = store.borrow::<ViewMut<Confusion>>().unwrap();
    let vactor = store.borrow::<View<Actor>>().unwrap();
    let mut vwantsattack = store.borrow::<ViewMut<WantsToAttack>>().unwrap();

    let mut needs_wants_to_attack: Vec<EntityId> = Vec::new();
    let mut to_update_confusion: Vec<(EntityId, Confusion)> = Vec::new();

    // Monster ai
    for (id, (actor, pos, vs)) in (&vactor, &vpos, &vvs).iter().with_id() {
        if actor.atype != ActorType::Orc {
            continue;
        }

        match vconfusion.get(id) {
            Err(_e) => {}
            Ok(confusion) => {
                to_update_confusion.push((id, *confusion));
                for pos in pos.ps.iter() {
                    particle_builder.request(
                        pos.x,
                        pos.y,
                        0.0,
                        0.0,
                        Palette::COLOR_3,
                        Palette::MAIN_BG,
                        rltk::to_cp437('?'),
                        300.0,
                    );
                }

                // TODO attempt to move in a random direction

                continue;
            }
        }

        // don't do anything if player is out of sight
        if !vs.visible_tiles.contains(&ppos) {
            continue;
        }

        // TODO mutlitile monsters currently only attack from their first position
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(ppos, pos.ps[0]);
        if distance < 1.5 {
            needs_wants_to_attack.push(id);
        } else if vs.visible_tiles.contains(&ppos) {
            // in order to stop multi-tile monsters from blocking themselves, make them not block before running A*
            // this is still just a hack since multi-tile monsters still path through 1 wide areas
            for pos in pos.ps.iter() {
                let idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
            }
            let path = utils::get_path(&map, pos.ps[0], ppos);

            // make monster block again
            for pos in pos.ps.iter() {
                let idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = true;
            }

            if path.success && path.steps.len() > 1 {
                add_effect(
                    Some(id),
                    EffectType::Move {
                        tile_idx: path.steps[1],
                    },
                )
            }
        }
    }

    for id in needs_wants_to_attack.iter() {
        vwantsattack.add_component_unchecked(*id, WantsToAttack { target: player_id });
    }

    for (id, _confusion) in to_update_confusion.iter() {
        let mut to_remove = false;
        {
            let mut c = *vconfusion.get(*id).unwrap();
            c.turns -= 1;
            if c.turns <= 0 {
                to_remove = true
            }
        }
        if to_remove {
            vconfusion.remove(*id);
        }
    }
}
