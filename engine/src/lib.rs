#[macro_use]
extern crate lazy_static;

pub mod components;
pub mod map;
pub mod utils;
pub mod map_builders;
pub mod entity_factory;
pub mod colors;
pub mod worldgen;
pub mod game_modes;
pub mod tiles;
pub mod systems;
pub mod effects;
pub mod ai;
pub mod player;
pub mod world_sim;

pub const SHOW_MAPGEN_ANIMATION: bool = true;
pub const MAPGEN_FRAME_TIME: f32 = 25.0;

pub const TILE_SIZE: usize = 10;
pub const SCALE: f32 = 1.0;

pub const OFFSET_X: usize = 31;
pub const OFFSET_Y: usize = 11;

pub const DISABLE_AI: bool = false;
pub const DISABLE_FOV: bool = true;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items = 0,
    NPC,
    Player,
    Particle,
}
