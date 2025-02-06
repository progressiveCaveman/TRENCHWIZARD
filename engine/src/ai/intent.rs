use serde::Serialize;
use shipyard::{Component, EntityId};

use crate::{simulator::components::{self, Turn}, utils::Target};

use super::input::InputTargets;

#[derive(Clone, Debug, Copy, PartialEq, Serialize)]
pub enum Task {
    Fish,    // not an effect yet but maybe could be?
    Explore, //
    ExchangeInfo,
    MoveTo(InputTargets),
    Destroy(InputTargets),
    PickUpItem(InputTargets), //
    DropItem,   //
    UseItem,
    EquipItem,
    UnequipItem,
    UseWorkshop,
    DepositItemToInventory(InputTargets, InputTargets),
    Attack(InputTargets),
    Idle,
    Spawn(InputTargets),
}

#[derive(Component, Clone, Debug)]
pub struct Intent {
    pub name: String,
    pub owner: EntityId,
    pub task: Task,
    pub target: Vec<Target>, // most tasks have one target, more targets are specified in name, ie `DepositItemToInventory` expects [item, inventory]
    pub turn: Turn,          // turn this intent originated
}

impl Intent {
    pub fn idle() -> Self {
        Intent {
            name: "Idle".to_string(),
            owner: EntityId::default(),
            task: Task::Idle,
            target: Vec::new(),
            turn: components::Turn(0),
        }
    }
}

// Actions are stored using archetype, and specific intents are generated on the fly
#[derive(Component, Clone, Debug, PartialEq, Serialize)]
pub struct IntentArchetype {
    pub name: String,
    pub task: Task,
}