use serde::{Serialize, Deserialize};

use crate::colors::{Color, self, ColorUtils};

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TileType {
    Water,
    Sand,
    Dirt,
    Stone,
    Wall,
    Floor,
    StairsDown,
    StairsUp,
    Grass,
    Wheat,
    WoodWall,
    WoodDoor,
    WoodFloor,
}

pub type TileRenderable = (char, Color, Color);

impl TileType {
    pub fn renderable(self) -> TileRenderable {
        match self {
            TileType::Water => ('~', colors::COLOR_WATER, colors::COLOR_WATER.scale(0.75)),
            TileType::Sand => ('.', colors::COLOR_SAND, colors::COLOR_SAND.scale(0.5)),
            TileType::Dirt => ('.', colors::COLOR_DIRT, colors::COLOR_BG),
            TileType::Stone => ('.', colors::COLOR_STONE, colors::COLOR_BG),
            TileType::Wall => ('#', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::Floor => ('.', colors::COLOR_FLOOR, colors::COLOR_BG),
            TileType::StairsDown => ('>', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::StairsUp => ('<', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::Grass => (',', colors::COLOR_GRASS, colors::COLOR_GRASS.scale(0.5)),
            TileType::Wheat => ('{', colors::COLOR_AMBER, colors::COLOR_BG),
            TileType::WoodWall => ('#', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
            TileType::WoodDoor => ('+', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
            TileType::WoodFloor => ('.', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
        }
    }
}

pub const MAX_GAS: usize = 7;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GasType {
    Air,
    Blocked,
    Steam,
    None,
}