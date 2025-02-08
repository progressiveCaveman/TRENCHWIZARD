use serde::Serialize;
use serde::Deserialize;
use crate::world::components::ItemType;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    Const, // used as a baseline for things
    DistanceTo(InputTargets),
    Inventory(InputTargets), // intent owner's inventory
    // TargetInventory(InputTargets), // target's inventory
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputTargets {
    Tree,
    Log,
    LumberMill,
    Water,
    Fishery,
    Enemy,
    Fish,
    Player,
    None,
    Orc,
}

impl InputTargets {
    pub fn matches(&self, item: ItemType) -> bool {
        match self {
            InputTargets::Log => item == ItemType::Log,
            InputTargets::Fish => item == ItemType::Fish,
            _ => false,
        }
    }
}