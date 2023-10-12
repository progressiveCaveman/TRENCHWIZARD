use serde::{Serialize, Deserialize};


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