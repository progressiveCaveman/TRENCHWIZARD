use std::collections::HashMap;

use rltk::{self, DijkstraMap, Point};
use serde::{Deserialize, Serialize};
use shipyard::{Component, EntityId, IntoIter, View, Unique, AllStorages, Get, UniqueView, IntoWithId};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

use crate::{
    map::Map,
    RenderOrder, tiles::{TileType, GasType}, colors::{COLOR_BG, Color}, ai::{labors::AIBehaviors, decisions::InputTargets}, utils::Target,
};

/// Unique components

#[derive(Debug, Unique)]
pub struct GameLog {
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Unique, Copy)]
pub struct PlayerID(pub EntityId);

#[derive(Clone, Debug, Unique, Copy)]
pub struct Turn(pub i32);

#[derive(Clone, Unique)]
pub struct RNG(pub rltk::RandomNumberGenerator);

#[derive(Clone, Debug, Unique, Copy)]
pub struct PPoint(pub Point);

#[derive(Clone, Debug, Unique, Copy)]
pub struct FrameTime(pub f32);

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
    pub glyph: char,
    pub fg: Color,
    pub bg: Color,
    pub render: bool,
    pub always_render: bool,
    pub order: RenderOrder,
}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            glyph: ' ',
            fg: COLOR_BG,
            bg: COLOR_BG,
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

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Orc {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Fish {}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Actor {
    pub atype: ActorType,
    pub faction: Faction,
    pub behaviors: Vec<AIBehaviors>,
    pub score: i32, // actors score points for executing behaviors
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

/// Entity properties

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub enum LocomotionType {
    Ground,
    Water,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Locomotive {
    pub mtype: LocomotionType,
    pub speed: i32,
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct BlocksTile {}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct PhysicalStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub regen_rate: i32,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SpatialKnowledge {
    pub tiles: HashMap<usize, (TileType, Vec<EntityId>)>,
}

impl SpatialKnowledge {
    pub fn get_targets(&self, store: &AllStorages, target: InputTargets) -> Vec<Target> {
        let map = store.borrow::<UniqueView<Map>>().unwrap();

        let mut targets = vec![];

        for (idx, (tile, entities)) in self.tiles.iter() {
            for id in entities {
                match target {
                    InputTargets::Tree => {
                        if let Ok(_) = store.borrow::<View<Tree>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                    InputTargets::Log => {
                        if let Ok(item) = store.borrow::<View<Item>>().unwrap().get(*id){
                            if item.typ == ItemType::Log {
                                targets.push(Target::from(*id));
                            }
                        }
                    },
                    InputTargets::LumberMill => {
                        if let Ok(_) = store.borrow::<View<LumberMill>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                    InputTargets::Water => {
                        if *tile == TileType::Water {
                            targets.push(Target::from(map.idx_point(*idx)));
                        }
                    },
                    InputTargets::Fishery => {
                        if let Ok(_) = store.borrow::<View<FishCleaner>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                    InputTargets::Enemy => todo!(),
                    InputTargets::Fish => {
                        if let Ok(_) = store.borrow::<View<Fish>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                    InputTargets::Player => {
                        if let Ok(_) = store.borrow::<View<Tree>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                    InputTargets::None => { },
                    InputTargets::Orc => {
                        if let Ok(_) = store.borrow::<View<Fish>>().unwrap().get(*id){
                            targets.push(Target::from(*id));
                        }
                    },
                }
            }
        }

        targets
    }
}

#[derive(Component)]
pub struct DijkstraMapToMe {
    pub map: DijkstraMap,
}

#[derive(Component)]
pub struct IsCamera {}

#[derive(Component)]
pub struct Aging {
    pub turns: i32,
}

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
#[derive(Component, Clone, Debug, PartialEq)]
pub struct AddsGas {
    pub gas: GasType,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct RemovesGas {
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Inventory {
    pub capacity: i32,
    pub items: Vec<EntityId>,
}

impl Inventory {
    pub fn count_type(&self, vitems: &View<Item>, item_type: ItemType) -> i32 {
        let mut count = 0;
        for (id, item) in vitems.iter().with_id() {
            if item.typ == item_type && self.items.contains(&id){
                count += 1;
            }
        }

        return count;
    }
}

#[derive(Component, PartialEq, Copy, Clone, Eq, Hash, Debug, EnumIter)]
pub enum EquipmentSlot {
    LeftHand,
    RightHand,
    Torso,
    Head,
    Legs,
    Feet,
    Back,
}

#[derive(Component, Clone, PartialEq)]
pub struct Equipment {
    pub items: HashMap<EquipmentSlot, Option<EntityId>>,
}

impl Equipment {
    pub fn new() -> Self {
        Self { 
            items: HashMap::from([
                (EquipmentSlot::LeftHand, None),
                (EquipmentSlot::RightHand, None),
                (EquipmentSlot::Torso, None),
                (EquipmentSlot::Head, None),
                (EquipmentSlot::Legs, None),
                (EquipmentSlot::Feet, None),
                (EquipmentSlot::Back, None),
            ]) 
        }
    }

    pub fn equip(&mut self, item: EntityId, slot: EquipmentSlot) {
        self.items.insert(slot, Some(item));
    }

    pub fn unequip(&mut self, id: EntityId) {
        for slot in EquipmentSlot::iter() {
            if let Some(item) = self.items.get(&slot).unwrap() {
                if *item == id {
                    self.items.insert(slot, None);
                }
            }
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component)]
pub struct Equipped { // todo is this necessary?
    pub owner: EntityId,
    pub slot: EquipmentSlot,
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
pub struct OnFire {
    pub turns: i32,
}

#[derive(Component, Clone, Copy)]
pub struct CausesFire {
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
