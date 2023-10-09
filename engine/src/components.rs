use std::collections::HashMap;

use rltk::{self, DijkstraMap, Point};
use serde::{Deserialize, Serialize};
use shipyard::{Component, EntityId, IntoIter, View};

use crate::{
    ai::labors::AIBehaviors,
    map::{Map, TileType},
    RenderOrder,
};

/// Basic UI components

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub ps: Vec<Point>,
}

impl Position {
    pub fn any_point(&self) -> Point {
        if self.ps.len() > 0 {
            *self.ps.first().unwrap()
        } else {
            unreachable!()
        }
    }

    pub fn idxes(&self, map: &Map) -> Vec<usize> {
        self.ps.iter().map(|it| map.point_idx(*it)).collect()
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGBA,
    pub bg: rltk::RGBA,
    pub render: bool,
    pub always_render: bool,
    pub order: RenderOrder,
}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            glyph: rltk::to_cp437(' '),
            fg: rltk::RGBA {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            bg: rltk::RGBA {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
            render: true,
            always_render: false,
            order: RenderOrder::Player,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Vision {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

impl Vision {
    pub fn is_visible(&self, idx: Point) -> bool {
        for p in self.visible_tiles.iter() {
            if p.x == idx.x && p.y == idx.y {
                return true;
            }
        }

        false
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String,
}

/// Entity properties

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Player {}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Actor {
    pub atype: ActorType,
    pub faction: Faction,
    pub behaviors: Vec<AIBehaviors>, // TODO instead of specifying, make a selector. Then give add copy back to this comp
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Faction {
    Nuetral,
    Nature,
    Player,
    Orcs,
    Villager,
    Wizard1,
    Wizard2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActorType {
    Player,
    Orc,
    Villager,
    Fish,
    Spawner,
    Wolf,
}

/// Structures

#[derive(Component, Clone, Debug, PartialEq)]
pub struct PlankHouse {
    pub housing_cap: i32,
    pub villagers: Vec<EntityId>,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ChiefHouse {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct LumberMill {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct FishCleaner {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum SpawnerType {
    Orc,
    Fish,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Spawner {
    pub typ: SpawnerType,
    pub rate: i32,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Tree {}

/// Labors?

/// Entity properties

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum LocomotionType {
    Ground,
    Water,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Locomotive {
    pub mtype: LocomotionType,
    pub speed: usize,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct BlocksTile {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub regen_rate: i32,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Inventory {
    pub capacity: i32,
    pub items: Vec<EntityId>,
}

impl Inventory {
    pub fn count_type(&self, vitems: &View<Item>, item_type: ItemType) -> i32 {
        let mut count = 0;
        for item in vitems.iter() {
            if item.typ == item_type {
                count += 1;
            }
        }

        return count;
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SpatialKnowledge {
    pub tiles: HashMap<usize, (TileType, Vec<EntityId>)>,
}

#[derive(Component)]
pub struct DijkstraMapToMe {
    pub map: DijkstraMap,
}

#[derive(Component)]
pub struct IsCamera {}

/// Entity intents

#[derive(Component, Clone, Copy)]
pub struct WantsToAttack {
    pub target: EntityId,
}

#[derive(Component, Clone, Copy)]
pub struct WantsToPickupItem {
    pub collected_by: EntityId,
    pub item: EntityId,
}

#[derive(Component, Clone, Copy)]
pub struct WantsToDropItem {
    pub item: EntityId,
}

#[derive(Component)]
pub struct WantsToUnequipItem {
    pub item: EntityId,
}

#[derive(Component)]
pub struct WantsToUseItem {
    pub item: EntityId,
    pub target: Option<rltk::Point>,
}

/// Inventory components

#[derive(Component, PartialEq, Copy, Clone)]
pub enum EquipmentSlot {
    RightHand,
    LeftHand,
}

#[derive(Component, Copy, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component)]
pub struct Equipped {
    pub owner: EntityId,
    pub slot: EquipmentSlot,
}

#[derive(Component)]
pub struct InBackpack {
    pub owner: EntityId,
}

/// Item properties

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum ItemType {
    Log,
    Shield,
    Weapon,
    Potion,
    Scroll,
    Fish,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Item {
    pub typ: ItemType,
}

#[derive(Component)]
pub struct Consumable {}

#[derive(Component)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Component)]
pub struct MeleeDefenseBonus {
    pub defense: i32,
}

#[derive(Component, Clone, Copy)]
pub struct ProvidesHealing {
    pub heal: i32,
}

#[derive(Component)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Clone, Copy)]
pub struct DealsDamage {
    pub damage: i32,
}

#[derive(Component, Clone, Copy)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component)]
pub struct AreaOfEffect {
    pub radius: i32,
}

/// Fire components

#[derive(Component, Clone, Copy)]
pub struct Fire {
    pub turns: i32,
}

#[derive(Component, Clone, Copy)]
pub struct Flammable {}

/// Save components

#[derive(Component)]
pub struct Lifetime {
    pub ms: f32,
}

/// Particle components

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Particle {
    pub float_x: f32,
    pub float_y: f32,
}
