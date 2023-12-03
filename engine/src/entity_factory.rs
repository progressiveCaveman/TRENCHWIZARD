use std::collections::HashMap;

use crate::ai::labors::AIBehaviors;
use crate::colors::{*};
// use crate::ai::labors::AIBehaviors;
use crate::components::{
    Actor, ActorType, AreaOfEffect, BlocksTile, ChiefHouse, PhysicalStats, Confusion, Consumable, DealsDamage,
    DijkstraMapToMe, EquipmentSlot, Equippable, Faction, FishCleaner, Flammable, Inventory, Item, ItemType,
    LocomotionType, Locomotive, LumberMill, MeleeDefenseBonus, MeleePowerBonus, Name, PlankHouse, Player, Position,
    ProvidesHealing, Ranged, Renderable, SpatialKnowledge, Spawner, SpawnerType, Tree, Vision, RNG, CausesFire, Equipment, AddsGas, RemovesGas, Aging,
};
use crate::map::{Map, XY};
// use crate::systems::system_fire::NEW_FIRE_TURNS;
// use crate::weighted_table::WeightedTable;
use crate::RenderOrder;
use crate::tiles::{TileType, GasType};
use crate::utils::rect::Rect;
use crate::utils::weighted_table::WeightedTable;
use rltk::{DijkstraMap, Point};
use shipyard::{AllStoragesViewMut, EntityId, UniqueView, UniqueViewMut};

const MAX_MONSTERS: usize = 4;

pub fn room_table(depth: usize) -> WeightedTable {
    WeightedTable::new()
        .add("Wolf", 10)
        .add("Goblin", 10)
        .add("Orc", 1 + depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + depth)
        .add("Confusion Scroll", 2 + depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 2)
        .add("Shield", 2)
        .add("Longsword", depth - 1)
        .add("Tower Shield", depth - 1)
}

pub fn spawn_room(store: &mut AllStoragesViewMut, map: &Map, room: &Rect, depth: usize) {
    let mut possible_targets: Vec<usize> = Vec::new();
    
    // Borrow scope - to keep access to the map separated
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let idx = map.xy_idx((x , y));
            if map.tiles[idx] == TileType::Floor {
                possible_targets.push(idx);
            }
        }
    }

    spawn_region(store, &possible_targets, depth);
}

pub fn spawn_region(store: &mut AllStoragesViewMut, area: &[usize], map_depth: usize) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let mut rng = &mut store.borrow::<UniqueViewMut<RNG>>().unwrap().0;

        let num_spawns = usize::min(
            areas.len() as usize,
            rng.roll_dice(1, MAX_MONSTERS as i32 + 3) as usize + map_depth - 1,
        );
        if num_spawns == 0 {
            return;
        }

        for _i in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0usize
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(&mut rng).unwrap());
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_entity(store, &spawn);
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
fn spawn_entity(store: &mut AllStoragesViewMut, spawn: &(&usize, &String)) {
    let xy = store.run(|map: UniqueView<Map>| map.idx_xy(*spawn.0));

    match spawn.1.as_ref() {
        "Wolf" => wolf(store, xy),
        "Goblin" => goblin(store, xy),
        "Orc" => orc(store, xy),
        "Health Potion" => health_potion(store, xy),
        "Fireball Scroll" => fireball_scroll(store, xy),
        "Confusion Scroll" => confusion_scroll(store, xy),
        "Magic Missile Scroll" => magic_missile_scroll(store, xy),
        "Dagger" => dagger(store, xy),
        "Shield" => shield(store, xy),
        "Longsword" => longsword(store, xy),
        "Tower Shield" => tower_shield(store, xy),
        _ => unreachable!(),
    };
}

#[derive(Debug, Clone, Copy)]
pub enum EntitySpawnTypes {
    Villager
}

pub fn spawn_entity_type(store: &mut AllStoragesViewMut, etype: EntitySpawnTypes, pos: XY) {
    match etype {
        EntitySpawnTypes::Villager => {
            villager(store, pos);
        },
    }
}

pub fn player(store: &mut AllStoragesViewMut, pos: XY, is_render: bool) -> EntityId {
    let e = store.add_entity((
        Position {
            ps: vec![Point::new(pos.0, pos.1)],
        },
        Renderable {
            glyph: '@',
            fg: if is_render { COLOR_PURPLE } else { COLOR_BG },
            bg: COLOR_BG,
            order: RenderOrder::Player,
            ..Default::default()
        },
        Player {},
        Actor {
            faction: Faction::Player,
            atype: ActorType::Player,
            behaviors: Vec::new(),
            score: 0,
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true,
        },
        Name {
            name: "Player".to_string(),
        },
        PhysicalStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
            regen_rate: 1,
        },
        SpatialKnowledge { tiles: HashMap::new() },
        Inventory {
            capacity: 20,
            items: Vec::new(),
        },
    ));

    store.add_component(e, (
        DijkstraMapToMe {
            map: DijkstraMap::new_empty(0, 0, 0.),
        },
        Equipment::new()
    ));

    e
}

/// Monsters

pub fn villager(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: 'v',
            fg: COLOR_RED,
            bg: COLOR_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true,
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name {
            name: "Villager".to_string(),
        },
        BlocksTile {},
        Inventory {
            capacity: 5,
            items: Vec::new(),
        },
        SpatialKnowledge { tiles: HashMap::new() },
        Actor {
            faction: Faction::Villager,
            atype: ActorType::Villager,
            behaviors: vec![AIBehaviors::GatherWood, AIBehaviors::GatherFish, AIBehaviors::Wander],
            score: 0,
        },
        Aging {
            turns: 0,
        }
    ))
}

pub fn fish(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: 'f',
            fg: COLOR_AMBER,
            bg: COLOR_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 2,
            dirty: true,
        },
        Locomotive {
            mtype: LocomotionType::Water,
            speed: 1,
        },
        Name {
            name: "Fish".to_string(),
        },
        Actor {
            faction: Faction::Nature,
            atype: ActorType::Fish,
            behaviors: Vec::new(),
            score: 0,
        },
        Item { typ: ItemType::Fish },
    ))
}

pub fn orc(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    monster(store, xy, 'o', "Orc".to_string())
}

pub fn goblin(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    monster(store, xy, 'g', "Goblin".to_string())
}

pub fn monster(store: &mut AllStoragesViewMut, xy: XY, glyph: char, name: String) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph,
            fg: COLOR_BROWN,
            bg: COLOR_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        Actor {
            faction: Faction::Orcs,
            atype: ActorType::Orc,
            behaviors: vec![AIBehaviors::AttackEnemies],
            score: 0,
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name { name },
        BlocksTile {},
        PhysicalStats {
            max_hp: 8,
            hp: 8,
            defense: 1,
            power: 4,
            regen_rate: 0,
        },
        Inventory {
            capacity: 5,
            items: Vec::new(),
        },
    ))
}

pub fn wolf(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: 'w',
            fg: COLOR_RED,
            bg: COLOR_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 5,
            dirty: true,
        },
        Actor {
            faction: Faction::Nature,
            atype: ActorType::Wolf,
            behaviors: vec![AIBehaviors::AttackEnemies],
            score: 0,
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name {
            name: "Wolf".to_string(),
        },
        BlocksTile {},
        PhysicalStats {
            max_hp: 8,
            hp: 8,
            defense: 1,
            power: 4,
            regen_rate: 1,
        },
    ))
}

#[allow(dead_code)]
pub fn big_monster(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![
                Point::new(xy.0, xy.1),
                Point::new(xy.0 + 1, xy.1),
                Point::new(xy.0, xy.1 + 1),
                Point::new(xy.0 + 1, xy.1 + 1),
            ],
        },
        Renderable {
            glyph: 'o',
            fg: COLOR_RED,
            bg: COLOR_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Vision {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
        Actor {
            faction: Faction::Orcs,
            atype: ActorType::Orc,
            behaviors: vec![AIBehaviors::AttackEnemies],
            score: 0,
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name {
            name: "Monster".to_string(),
        },
        BlocksTile {},
        PhysicalStats {
            max_hp: 8,
            hp: 8,
            defense: 1,
            power: 4,
            regen_rate: 0,
        },
    ))
}

/// consumables

const WEAPON_GLYPH: char = '/';
const ARMOR_GLYPH: char = '{';
const POTION_GLYPH: char = '!';
const SCROLL_GLYPH: char = '?';
const LOG_GLYPH: char = '=';

pub fn health_potion(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: POTION_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Health potion".to_string(),
        },
        Item { typ: ItemType::Potion },
        ProvidesHealing { heal: 8 },
        Consumable {},
    ))
}

pub fn magic_missile_scroll(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: SCROLL_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Magic missile scroll".to_string(),
        },
        Item { typ: ItemType::Scroll },
        Consumable {},
        DealsDamage { damage: 8 },
        Ranged { range: 6 },
    ))
}

pub fn fireball_scroll(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: SCROLL_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Fireball scroll".to_string(),
        },
        Item { typ: ItemType::Scroll },
        Consumable {},
        DealsDamage { damage: 20 },
        Ranged { range: 6 },
        AreaOfEffect { radius: 3 },
        CausesFire { turns: 5 }
    ))
}

pub fn confusion_scroll(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: SCROLL_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Confusion scroll".to_string(),
        },
        Item { typ: ItemType::Scroll },
        Consumable {},
        Ranged { range: 6 },
        Confusion { turns: 4 },
    ))
}

/// equippables

pub fn dagger(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: WEAPON_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Dagger".to_string(),
        },
        Item { typ: ItemType::Weapon },
        Equippable {
            slot: EquipmentSlot::RightHand,
        },
        MeleePowerBonus { power: 4 },
    ))
}

pub fn longsword(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: WEAPON_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "longsword".to_string(),
        },
        Item { typ: ItemType::Shield },
        Equippable {
            slot: EquipmentSlot::RightHand,
        },
        MeleePowerBonus { power: 8 },
    ))
}

pub fn shield(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: ARMOR_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Shield".to_string(),
        },
        Item { typ: ItemType::Shield },
        Equippable {
            slot: EquipmentSlot::LeftHand,
        },
        MeleeDefenseBonus { defense: 4 },
    ))
}

pub fn tower_shield(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: ARMOR_GLYPH,
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Shield".to_string(),
        },
        Item { typ: ItemType::Shield },
        Equippable {
            slot: EquipmentSlot::LeftHand,
        },
        MeleeDefenseBonus { defense: 8 },
    ))
}

pub fn log(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: LOG_GLYPH,
            fg: COLOR_CEDAR,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Log".to_string(),
        },
        Item { typ: ItemType::Log },
        Flammable {},
    ))
}

// structures

pub fn gas_adder(store: &mut AllStoragesViewMut, xy: XY, gastype: GasType) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: '=',
            fg: COLOR_WALL.scale(0.25),
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Gas Vent".to_string(),
        },
        AddsGas { gas: gastype },
    ))
}

pub fn gas_remover(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: '=',
            fg: COLOR_WALL.scale(0.25),
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Gas Intake".to_string(),
        },
        RemovesGas {},
    ))
}

pub fn spawner(
    store: &mut AllStoragesViewMut,
    xy: XY,
    faction: Faction,
    typ: SpawnerType,
    rate: i32,
) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: '&',
            fg: COLOR_CHARTREUSE,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Spawner".to_string(),
        },
        Spawner { typ, rate },
        Actor {
            atype: ActorType::Spawner,
            faction,
            behaviors: Vec::new(),
            score: 0,
        },
    ))
}

pub fn tree(store: &mut AllStoragesViewMut, xy: XY) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( xy.0, xy.1 )],
        },
        Renderable {
            glyph: '|',
            fg: COLOR_CEDAR,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Tree".to_string(),
        },
        Flammable {},
        Tree {},
    ))
}

pub fn plank_house(store: &mut AllStoragesViewMut, xy: XY, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( xy.0 + xi as i32, xy.1 + yi as i32));
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    store.add_entity((
        Position { ps },
        Renderable {
            glyph: '#',
            fg: COLOR_CEDAR,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Plank House".to_string(),
        },
        Flammable {},
        PlankHouse {
            housing_cap: 5,
            villagers: vec![],
        },
        BlocksTile {},
    ))
}

pub fn chief_house(store: &mut AllStoragesViewMut, xy: XY, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new(xy.0 + xi as i32, xy.1 + yi as i32));
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    store.add_entity((
        Position { ps },
        Renderable {
            glyph: '#',
            fg: COLOR_CEDAR,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "chief_house".to_string(),
        },
        Flammable {},
        ChiefHouse {},
        BlocksTile {},
    ))
}

pub fn fish_cleaner(store: &mut AllStoragesViewMut, xy: XY, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( xy.0 + xi as i32, xy.1 + yi as i32));
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    store.add_entity((
        Position { ps },
        Renderable {
            glyph: '#',
            fg: COLOR_UI_1,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Fish Cleaner".to_string(),
        },
        Flammable {},
        FishCleaner {},
        BlocksTile {},
        Inventory {
            capacity: 50,
            items: Vec::new(),
        },
        DijkstraMapToMe {
            map: DijkstraMap::new_empty(0, 0, 0.),
        },
    ))
}

pub fn lumber_mill(store: &mut AllStoragesViewMut, xy: XY, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( xy.0 + xi as i32, xy.1 + yi as i32));
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    store.add_entity((
        Position { ps },
        Renderable {
            glyph: '#',
            fg: COLOR_AMBER,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Lumber Mill".to_string(),
        },
        Flammable {},
        LumberMill {},
        BlocksTile {},
        Inventory {
            capacity: 50,
            items: Vec::new(),
        },
        DijkstraMapToMe {
            map: DijkstraMap::new_empty(0, 0, 0.),
        },
    ))
}

/// misc

pub fn tmp_fireball(store: &mut AllStoragesViewMut) -> EntityId {
    store.add_entity((
        Name {
            name: "Fireball".to_string(),
        },
        Item { typ: ItemType::Scroll },
        Consumable {},
        DealsDamage { damage: 20 },
        Ranged { range: 6 },
        AreaOfEffect { radius: 1 },
        CausesFire { turns: 3 },
    ))
}
