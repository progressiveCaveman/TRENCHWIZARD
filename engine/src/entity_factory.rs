use std::collections::HashMap;

use crate::colors::{*};
// use crate::ai::labors::AIBehaviors;
use crate::components::{
    Actor, ActorType, AreaOfEffect, BlocksTile, ChiefHouse, CombatStats, Confusion, Consumable, DealsDamage,
    DijkstraMapToMe, EquipmentSlot, Equippable, Faction, Fire, FishCleaner, Flammable, Inventory, Item, ItemType,
    LocomotionType, Locomotive, LumberMill, MeleeDefenseBonus, MeleePowerBonus, Name, PlankHouse, Player, Position,
    ProvidesHealing, Ranged, Renderable, SpatialKnowledge, Spawner, SpawnerType, Tree, Vision, RNG,
};
use crate::map::Map;
// use crate::systems::system_fire::NEW_FIRE_TURNS;
// use crate::weighted_table::WeightedTable;
use crate::RenderOrder;
use crate::tiles::TileType;
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
    {
        // Borrow scope - to keep access to the map separated
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx((x as usize, y as usize));
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
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
            rng.roll_dice(1, MAX_MONSTERS as i32 + 3) as usize + (map_depth - 1) - 3,
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
    let (x, y) = store.run(|map: UniqueView<Map>| map.idx_xy(*spawn.0));

    match spawn.1.as_ref() {
        "Wolf" => wolf(store, x, y),
        "Goblin" => goblin(store, x, y),
        "Orc" => orc(store, x, y),
        "Health Potion" => health_potion(store, x, y),
        "Fireball Scroll" => fireball_scroll(store, x, y),
        "Confusion Scroll" => confusion_scroll(store, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(store, x, y),
        "Dagger" => dagger(store, x, y),
        "Shield" => shield(store, x, y),
        "Longsword" => longsword(store, x, y),
        "Tower Shield" => tower_shield(store, x, y),
        _ => unreachable!(),
    };
}

pub fn player(store: &mut AllStoragesViewMut, pos: (usize, usize)) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new(pos.0, pos.1)],
        },
        Renderable {
            glyph: '@',
            fg: COLOR_PURPLE,
            bg: COLOR_BG,
            order: RenderOrder::Player,
            ..Default::default()
        },
        Player {},
        Actor {
            faction: Faction::Player,
            atype: ActorType::Player,
            // //behaviors: Vec::new(),
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
        CombatStats {
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
    ))
}

/// Monsters

pub fn villager(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
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
            //behaviors: vec![AIBehaviors::GatherWood, AIBehaviors::GatherFish, AIBehaviors::Wander],
        },
    ))
}

pub fn fish(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
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
            //behaviors: Vec::new(),
        },
        Item { typ: ItemType::Fish },
    ))
}

pub fn orc(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    monster(store, x, y, 'o', "Orc".to_string())
}

pub fn goblin(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    monster(store, x, y, 'g', "Goblin".to_string())
}

pub fn monster(store: &mut AllStoragesViewMut, x: usize, y: usize, glyph: char, name: String) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph,
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
            //behaviors: vec![AIBehaviors::AttackEnemies],
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name { name },
        BlocksTile {},
        CombatStats {
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

pub fn wolf(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
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
            //behaviors: vec![AIBehaviors::AttackEnemies],
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name {
            name: "Wolf".to_string(),
        },
        BlocksTile {},
        CombatStats {
            max_hp: 8,
            hp: 8,
            defense: 1,
            power: 4,
            regen_rate: 1,
        },
    ))
}

#[allow(dead_code)]
pub fn big_monster(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![
                Point::new(x, y),
                Point::new(x + 1, y),
                Point::new(x, y + 1),
                Point::new(x + 1, y + 1),
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
            //behaviors: vec![AIBehaviors::AttackEnemies],
        },
        Locomotive {
            mtype: LocomotionType::Ground,
            speed: 1,
        },
        Name {
            name: "Monster".to_string(),
        },
        BlocksTile {},
        CombatStats {
            max_hp: 8,
            hp: 8,
            defense: 1,
            power: 4,
            regen_rate: 0,
        },
    ))
}

/// consumables

pub fn health_potion(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: 'p',
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

pub fn magic_missile_scroll(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '(',
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

pub fn fireball_scroll(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '*',
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
    ))
}

pub fn confusion_scroll(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '&',
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

pub fn dagger(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '│',
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

pub fn longsword(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '│',
            fg: COLOR_ITEM,
            bg: COLOR_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {
            name: "Dagger".to_string(),
        },
        Item { typ: ItemType::Shield },
        Equippable {
            slot: EquipmentSlot::RightHand,
        },
        MeleePowerBonus { power: 8 },
    ))
}

pub fn shield(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '°',
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

pub fn tower_shield(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '°',
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

pub fn log(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '=',
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

pub fn spawner(
    store: &mut AllStoragesViewMut,
    x: usize,
    y: usize,
    faction: Faction,
    typ: SpawnerType,
    rate: usize,
) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
        },
        Renderable {
            glyph: '&',
            fg: FACTION_COLORS[faction as usize],
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
            //behaviors: Vec::new(),
        },
    ))
}

pub fn tree(store: &mut AllStoragesViewMut, x: usize, y: usize) -> EntityId {
    store.add_entity((
        Position {
            ps: vec![Point::new( x, y )],
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

pub fn plank_house(store: &mut AllStoragesViewMut, x: usize, y: usize, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( x + xi, y + yi));
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

pub fn chief_house(store: &mut AllStoragesViewMut, x: usize, y: usize, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new(x + xi, y + yi));
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

pub fn fish_cleaner(store: &mut AllStoragesViewMut, x: usize, y: usize, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( x + xi, y + yi));
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

pub fn lumber_mill(store: &mut AllStoragesViewMut, x: usize, y: usize, width: usize, height: usize) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point::new( x + xi, y + yi));
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
        AreaOfEffect { radius: 3 },
        Fire { turns: 5 },
    ))
}
