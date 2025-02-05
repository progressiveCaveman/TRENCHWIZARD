use crate::simulator::components::{PhysicalStats, Position, OnFire};
use crate::simulator::effects::{add_effect, EffectType, Targets};
use crate::map::Map;
use crate::tiles::TileType;
use crate::utils::InvalidPoint;
use rltk::RandomNumberGenerator;
use shipyard::{EntityId, IntoIter, IntoWithId, Remove, UniqueViewMut, ViewMut};

pub const NEW_FIRE_TURNS: i32 = 10;
pub const MAX_FIRE: i32 = 10;

pub fn run_fire_system(
    mut map: UniqueViewMut<Map>,
    vpos: ViewMut<Position>,
    vstats: ViewMut<PhysicalStats>,
    mut vonfire: ViewMut<OnFire>,
) {
    let mut rng = RandomNumberGenerator::new();

    // damage all entities on fire. If they are standing somewhere flammable, ignite it
    for (id, (pos, _, fire)) in (&vpos, &vstats, &vonfire).iter().with_id() {
        add_effect(
            None,
            EffectType::Damage {
                amount: 1,
                target: Targets::Single { target: id },
            },
        );

        for pos in pos.ps.iter() {
            let idx = map.xy_idx(pos.to_xy());
            if map.is_flammable(idx) && map.fire_turns[idx] == 0 {
                map.fire_turns[idx] = fire.turns;
            }
        }
    }

    // reduce fire turns and remove expired fire components
    let mut to_remove: Vec<EntityId> = vec![];
    (&mut vonfire).iter().with_id().for_each(|(id, fire)| {
        fire.turns = i32::min(fire.turns - 1, MAX_FIRE);

        if fire.turns <= 0 {
            to_remove.push(id);
            // vfire.remove(id);
        }
    });
    for e in to_remove.iter() {
        vonfire.remove(*e);
    }

    for idx in 0..(map.size.0 * map.size.1) as usize {
        if map.fire_turns[idx] > 0 {
            // reduce fire turns on tiles
            map.fire_turns[idx] -= 1;

            if map.fire_turns[idx] == 0 && map.is_flammable(idx) {
                map.tiles[idx] = TileType::Dirt;
            }

            // light entities on this tile on fire
            for e in map.tile_content[idx].iter() {
                add_effect(
                    None,
                    EffectType::Fire {
                        turns: map.fire_turns[idx],
                        target: Targets::Single { target: *e },
                    },
                );
            }

            // Chance to spread to nearby tiles
            let (x, y) = map.idx_xy(idx);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if map.in_bounds((nx, ny)) {
                        let idx = map.xy_idx((nx, ny));
                        if map.fire_turns[idx] == 0 && map.is_flammable(idx) && rng.range(0, 100) == 0 {
                            map.fire_turns[idx] = NEW_FIRE_TURNS;
                        }
                    }
                }
            }
        }
    }
}
