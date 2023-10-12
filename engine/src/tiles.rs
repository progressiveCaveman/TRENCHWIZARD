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
            TileType::Water => ('~', colors::COLOR_CYAN, colors::COLOR_WATER),
            TileType::Sand => ('.', colors::COLOR_SAND, colors::COLOR_BG),
            TileType::Dirt => ('.', colors::COLOR_DIRT, colors::COLOR_BG),
            TileType::Stone => ('.', colors::COLOR_STONE, colors::COLOR_BG),
            TileType::Wall => ('#', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::Floor => ('.', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::StairsDown => ('>', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::StairsUp => ('<', colors::COLOR_WALL, colors::COLOR_BG),
            TileType::Grass => (',', colors::COLOR_GREEN, colors::COLOR_GREEN.scale(0.5)),
            TileType::Wheat => ('{', colors::COLOR_AMBER, colors::COLOR_BG),
            TileType::WoodWall => ('#', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
            TileType::WoodDoor => ('+', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
            TileType::WoodFloor => ('.', colors::COLOR_DARKEST_AMBER, colors::COLOR_BG),
        }
    }
}