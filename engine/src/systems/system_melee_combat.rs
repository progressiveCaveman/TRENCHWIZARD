use crate::colors::{COLOR_BG, COLOR_UI_4};
use crate::components::GameLog;
use crate::effects::{EffectType, Targets};
use crate::{
    components::{PhysicalStats, Equipped, MeleeDefenseBonus, MeleePowerBonus, Name, Position, WantsToAttack},
    effects::add_effect,
    systems::system_particle::ParticleBuilder,
};
use shipyard::{AllStoragesView, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut};

pub fn run_melee_combat_system(store: AllStoragesView) {
    let mut log = store.borrow::<UniqueViewMut<GameLog>>().unwrap();
    let mut particle_builder = store.borrow::<UniqueViewMut<ParticleBuilder>>().unwrap();

    let mut vwants = store.borrow::<ViewMut<WantsToAttack>>().unwrap();
    let vname = store.borrow::<View<Name>>().unwrap();
    let vstats = store.borrow::<View<PhysicalStats>>().unwrap();
    let vmeleepower = store.borrow::<View<MeleePowerBonus>>().unwrap();
    let vmeleedefense = store.borrow::<View<MeleeDefenseBonus>>().unwrap();
    let vequipped = store.borrow::<View<Equipped>>().unwrap();
    let vpos = store.borrow::<View<Position>>().unwrap();

    let mut to_remove_wants_melee: Vec<EntityId> = vec![];

    for (id, (wants_attack, name, stats)) in (&vwants, &vname, &vstats).iter().with_id() {
        //&mut world.query::<(&WantsToAttack, &Name, &CombatStats)>() {
        if stats.hp > 0 {
            let target_stats = vstats.get(wants_attack.target).unwrap();
            if target_stats.hp > 0 {
                let mut offensize_bonus = 0;
                for (_item_id, (power_bonus, equipped)) in (&vmeleepower, &vequipped).iter().with_id() {
                    //.query::<(&MeleePowerBonus, &Equipped)>().iter() {
                    if equipped.owner == id {
                        offensize_bonus += power_bonus.power
                    }
                }

                if target_stats.hp > 0 {
                    let mut defensize_bonus = 0;
                    for (_item_id, (defense_bonus, equipped)) in (&vmeleedefense, &vequipped).iter().with_id() {
                        //world.query::<(&MeleeDefenseBonus, &Equipped)>().iter() {
                        if equipped.owner == wants_attack.target {
                            defensize_bonus += defense_bonus.defense
                        }
                    }
                    let damage = i32::max(
                        0,
                        (stats.power + offensize_bonus) - (target_stats.defense + defensize_bonus),
                    );

                    let target_name = vname.get(wants_attack.target).unwrap();
                    if damage == 0 {
                        log.messages
                            .push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    } else {
                        log.messages
                            .push(format!("{} hits {} for {} hp", &name.name, &target_name.name, damage));
                        add_effect(
                            Some(id),
                            EffectType::Damage {
                                amount: damage,
                                target: Targets::Single {
                                    target: wants_attack.target,
                                },
                            },
                        );
                    }

                    if let Ok(pos) = vpos.get(wants_attack.target) {
                        for pos in pos.ps.iter() {
                            particle_builder.request(
                                pos.x,
                                pos.y,
                                0.0,
                                0.0,
                                COLOR_UI_4,
                                COLOR_BG,
                                '‼',
                                500.0,
                            );
                        }
                    }
                }
            }
        }
        to_remove_wants_melee.push(id);
    }

    for id in to_remove_wants_melee.iter() {
        // let _res = world.remove_one::<WantsToAttack>(*id);
        vwants.remove(*id);
    }
}
