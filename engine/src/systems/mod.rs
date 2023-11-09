use shipyard::World;

use crate::{effects, DISABLE_AI};

pub mod system_ai;
pub mod system_ai_fish;
pub mod system_cleanup;
pub mod system_dissasemble;
pub mod system_fire;
pub mod system_map_indexing;
pub mod system_melee_combat;
pub mod system_particle;
pub mod system_pathfinding;
pub mod system_visibility;
pub mod system_inventory;
pub mod system_item_use;

pub fn run_systems(world: &mut World, _player_turn: bool, ai_turn: bool) {
    // if player_turn {
    world.run(system_fire::run_fire_system);
    // }

    world.run(effects::run_effects_queue);
    world.run(system_map_indexing::run_map_indexing_system);

    if ai_turn && !DISABLE_AI {
        world.run(system_pathfinding::run_pathfinding_system);
        world.run(system_ai_fish::run_fish_ai);
        world.run(system_ai::run_ai_system);
    }

    world.run(effects::run_effects_queue);

    world.run(system_melee_combat::run_melee_combat_system);
    world.run(system_inventory::run_inventory_system);
    world.run(system_dissasemble::run_dissasemble_system);
    world.run(system_inventory::run_drop_item_system);
    world.run(system_inventory::run_unequip_item_system);
    world.run(system_item_use::run_item_use_system);
    world.run(system_particle::spawn_particles);

    world.run(effects::run_effects_queue);
    world.run(system_map_indexing::run_map_indexing_system);
    
    world.run(system_visibility::run_visibility_system);
    world.run(system_cleanup::run_cleanup_system);
}