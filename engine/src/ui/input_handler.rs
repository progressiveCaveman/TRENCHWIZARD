use crate::{
    simulator::components::{Item, PlayerID, Inventory, PPoint, WantsToUseItem, Ranged, PhysicalStats, Position, Actor},
    simulator::effects::{add_effect, EffectType},
    simulator::map::{Map, to_point},
    utils::{dir_to_point, InvalidPoint, dir_to_offset}, game_modes::{GameMode, get_settings}, player, entity_factory,
};
use rltk::DistanceAlg;
use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, ViewMut, IntoIter, IntoWithId, AllStoragesViewMut};
use winit::event::{WindowEvent, VirtualKeyCode, ElementState};

use crate::{ui::screen::menu_config::{MainMenuSelection, ModeSelectSelection}, game::{Game, GameState}};


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InputCommand {
    None,
    Move { dir: i32 },
    ShowInventory,
    Wait,
    Escape,
    Get,
    Explore,
    RevealMap,
    Fireball,
    UseStairs,
    Pause,
    Apply, 
    Drop,

    //debug
    Reset,
    PrintAIParams,

    //ui
    ZoomIn,
    ZoomOut,
    Enter,
    Tab,
}

impl InputCommand {
    fn execute(&self, game: &mut Game, creator: Option<EntityId>) -> GameState {
        let world = &game.world_sim.world;

        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;

        // return GameState::None to ignore input, GameState::PlayerTurn to advance engine
        return match self {
            InputCommand::None => GameState::None,
            InputCommand::Move { dir } => {
                if game.world_sim.settings.mode == GameMode::MapDemo {
                    game.screen.pan_map(dir_to_offset(*dir).to_xy());
                }

                let updown = if *dir == 8 { -1 } else if *dir == 2 { 1 } else { 0 };
                match game.state {
                    GameState::MainMenu { selection } => return GameState::MainMenu { selection: selection.modify(updown) },
                    GameState::ModeSelect { selection } => return GameState::ModeSelect { selection: selection.modify(updown) },
                    GameState::ShowInventory { selection } => {
                        let newselection = selection as i32 + updown;
                        if let Ok(inv) = game.world_sim.world.borrow::<View<Inventory>>().unwrap().get(player_id) {
                            if newselection < 0 {
                                return GameState::ShowInventory { selection: inv.items.len() }
                            } else if newselection as usize >= inv.items.len() {
                                return GameState::ShowInventory { selection: 0 }
                            } else {
                                return GameState::ShowInventory { selection: newselection as usize}
                            }
                        }
                    },
                    _ => {},
                };

                let tile_idx = game.world_sim.get_map().point_idx(dir_to_point(player_pos, *dir, 1));

                add_effect(creator, EffectType::MoveOrAttack {tile_idx});

                GameState::PlayerActed
            }
            InputCommand::ShowInventory => GameState::ShowInventory { selection: 0 },
            InputCommand::Wait => {
                add_effect(creator, EffectType::Wait {}); //todo is this weird on sim mode?
                GameState::PlayerActed
            }
            InputCommand::Escape => {
                match game.state {
                    GameState::MainMenu { .. } => GameState::Exit,
                    GameState::ShowInventory { .. } | GameState::ShowItemActions { .. } | GameState::ShowTargeting { .. } => GameState::PreTurn,
                    _ => GameState::MainMenu { selection: MainMenuSelection::Play },
                }
            },
            InputCommand::Get => {
                world.run(|vitem: View<Item>, map: UniqueView<Map>| {
                    let player_pos_idx = map.point_idx(player_pos);
                    for e in map.tile_content[player_pos_idx].iter() {
                        if let Ok(_) = vitem.get(*e) {
                            add_effect(creator, EffectType::PickUp { entity: *e });
                        }
                    }
                });

                GameState::PlayerActed
            }
            InputCommand::Drop => {
                let item = match game.state {
                    GameState::ShowInventory { selection } => {
                        if let Ok(inv) = world.borrow::<View<Inventory>>().unwrap().get(player_id) {
                            Some(inv.items[selection])
                        } else {
                            None
                        }
                    },
                    GameState::ShowItemActions { item } => {
                        Some(item)
                    },
                    _ => None
                };

                if let Some(item) = item {
                    add_effect(creator, EffectType::Drop { entity: item });
                }

                GameState::PlayerActed
            },
            InputCommand::Explore => {
                add_effect(creator, EffectType::Explore {});

                GameState::PlayerActed
            }
            InputCommand::RevealMap => {
                player::reveal_map(&world);

                GameState::PlayerActed
            }
            InputCommand::Fireball => {
                let fireball = entity_factory::tmp_fireball(&mut game.world_sim.world.borrow::<AllStoragesViewMut>().unwrap());

                return GameState::ShowTargeting {
                    range: 6,
                    item: fireball,
                    target: (0,0)
                };
            }
            InputCommand::UseStairs => {
                // if player::try_next_level(&world) {
                //     GameState::NextLevel
                // } else {
                //     GameState::AwaitingInput
                // }
                GameState::None
            }
            InputCommand::Reset => {
                game.reset(None);
                GameState::ShowMapHistory
            },
            InputCommand::ZoomIn => {
                game.screen.increment_zoom();
                GameState::None
            },
            InputCommand::ZoomOut => {
                game.screen.decrement_zoom();
                GameState::None
            },
            InputCommand::Enter => {
                match game.state {
                    GameState::MainMenu { selection } => {
                        match selection {
                            MainMenuSelection::Play => GameState::ShowMapHistory,
                            MainMenuSelection::ModeSelect => GameState::ModeSelect { selection: ModeSelectSelection::from_repr(0).unwrap() },
                            MainMenuSelection::Quit => GameState::Exit,
                        }
                    },
                    GameState::ModeSelect { selection } => {
                        game.screen.reset();
                        match selection {
                            ModeSelectSelection::MapDemo => game.reset(Some(get_settings(GameMode::MapDemo))),
                            ModeSelectSelection::RL => game.reset(Some(get_settings(GameMode::RL))),
                            ModeSelectSelection::VillageSim => game.reset(Some(get_settings(GameMode::VillageSim))),
                            ModeSelectSelection::OrcArena => game.reset(Some(get_settings(GameMode::OrcArena))),
                        }

                        GameState::ShowMapHistory
                    },
                    GameState::ShowInventory { selection } => {
                        if let Ok(inv) = world.borrow::<View<Inventory>>().unwrap().get(player_id) {
                            if selection < inv.items.len() {
                                return GameState::ShowItemActions { item: inv.items[selection] };
                            }
                        }
                        GameState::None
                    },
                    GameState::ShowTargeting { range, item, target } => {
                        // if target is valid use item
                        if DistanceAlg::Pythagoras.distance2d(to_point(target), player_pos) < range as f32 {
                            game.world_sim.world.add_component(player_id, WantsToUseItem { item, target: Some(to_point(target)) });
                            return GameState::PlayerActed;
                        }

                        GameState::PreTurn
                    },
                    _ => GameState::None,
                }
            },
            InputCommand::Pause => {
                game.autorun = !game.autorun;
                GameState::None
            },
            InputCommand::Apply => {
                let item = match game.state {
                    GameState::ShowInventory { selection } => {
                        if let Ok(inv) = world.borrow::<View<Inventory>>().unwrap().get(player_id) {
                            Some(inv.items[selection])
                        } else {
                            None
                        }
                    },
                    GameState::ShowItemActions { item } => {
                        Some(item)
                    },
                    _ => None
                };

                if let Some(item) = item {
                    let mut to_add_wants_use_item: Vec<EntityId> = Vec::new();
                    {
                        let vranged: ViewMut<'_, Ranged, shipyard::track::Untracked> = game.world_sim.world.borrow::<ViewMut<Ranged>>().unwrap();
                        match vranged.get(item) {
                            Ok(is_item_ranged) => {
                                return GameState::ShowTargeting {
                                    range: is_item_ranged.range,
                                    item: item,
                                    target: (0,0) // todo pick closest target. Break this and tab code into a function
                                };
                            }
                            Err(_) => {
                                to_add_wants_use_item.push(player_id);
                            }
                        }
                    }
    
                    for id in to_add_wants_use_item.iter() {
                        game.world_sim.world.add_component(*id, WantsToUseItem { item, target: None });

                        return GameState::PlayerActed;
                    }
                }

                return GameState::None;
            },
            InputCommand::Tab => {
                match game.state {
                    GameState::ShowTargeting { range, item, target } => {
                        // get nearby units with stats, index through them, when target is found select the next one. Kind of a hack but maybe it works
                        let vstats = game.world_sim.world.borrow::<ViewMut<PhysicalStats>>().unwrap();
                        let vpos = game.world_sim.world.borrow::<ViewMut<Position>>().unwrap();

                        let mut targetfound = false;
                        let mut firsttarget = (0, 0);
                        for (id, (_, pos)) in (&vstats, &vpos).iter().with_id() {
                            if id == player_id {
                                continue;
                            }

                            let distance = DistanceAlg::Pythagoras.distance2d(pos.ps[0], player_pos);
                            if distance < range as f32 {
                                if firsttarget == (0,0) {
                                    firsttarget = pos.ps[0].to_xy();
                                }
                                for p in pos.ps.iter() {
                                    if targetfound {
                                        return GameState::ShowTargeting { range, item, target: pos.ps[0].to_xy() };
                                    }
                                    if p == &to_point(target) {
                                        targetfound = true;
                                    }
                                }
                            }
                        }

                        return GameState::ShowTargeting { range, item, target: firsttarget };
                    },
                    _ => game.state
                }
            },
            InputCommand::PrintAIParams => {
                let vactor = game.world_sim.world.borrow::<ViewMut<Actor>>().unwrap();
                println!("=======================================");
                println!("Printing villager params: ");
                for (_id, actor) in (&vactor).iter().with_id() {
                    let serialized = serde_json::to_string(&actor.actions).unwrap();
                    // println!("{:?}", serialized);
                    println!("serialized = {}", serialized);
                }
                GameState::None
            },
        };
    }
}

pub fn map_keys(event: WindowEvent, game: &Game) -> InputCommand {
    //universal commands
    if let WindowEvent::KeyboardInput { input,.. } = event {
        if input.state == ElementState::Pressed {
            let mut cmd = match input.virtual_keycode {
                None => InputCommand::None,
                Some(key) => match key {
                    VirtualKeyCode::Key1 => InputCommand::Move { dir: 1 },
                    VirtualKeyCode::Key2 => InputCommand::Move { dir: 2 },
                    VirtualKeyCode::Key3 => InputCommand::Move { dir: 3 },
                    VirtualKeyCode::Key4 => InputCommand::Move { dir: 4 },
                    VirtualKeyCode::Key5 => InputCommand::Wait,
                    VirtualKeyCode::Key6 => InputCommand::Move { dir: 6 },
                    VirtualKeyCode::Key7 => InputCommand::Move { dir: 7 },
                    VirtualKeyCode::Key8 => InputCommand::Move { dir: 8 },
                    VirtualKeyCode::Key9 => InputCommand::Move { dir: 9 },
                    VirtualKeyCode::Left => InputCommand::Move { dir: 4 },
                    VirtualKeyCode::Right => InputCommand::Move { dir: 6 },
                    VirtualKeyCode::Up => InputCommand::Move { dir: 8 },
                    VirtualKeyCode::Down => InputCommand::Move { dir: 2 },
                    VirtualKeyCode::F => InputCommand::Fireball,
                    VirtualKeyCode::W => InputCommand::Wait,
                    VirtualKeyCode::R => InputCommand::Reset,
                    VirtualKeyCode::P => InputCommand::PrintAIParams,
                    VirtualKeyCode::Return => InputCommand::Enter,
                    VirtualKeyCode::NumpadEnter => InputCommand::Enter,
                    VirtualKeyCode::Escape => InputCommand::Escape,
                    VirtualKeyCode::Equals => InputCommand::ZoomIn,
                    VirtualKeyCode::Minus => InputCommand::ZoomOut,
                    VirtualKeyCode::Tab => InputCommand::Tab,
                    _ => InputCommand::None,
                },
            };
    
            if cmd == InputCommand::None {
                cmd = match game.world_sim.settings.mode {
                    GameMode::RL | GameMode::OrcHalls => match input.virtual_keycode {
                        None => InputCommand::None,
                        Some(key) => match key {
                            VirtualKeyCode::G => InputCommand::Get,
                            VirtualKeyCode::X => InputCommand::Explore,
                            VirtualKeyCode::R => InputCommand::RevealMap,
                            VirtualKeyCode::I => InputCommand::ShowInventory,
                            VirtualKeyCode::A => InputCommand::Apply,
                            VirtualKeyCode::D => InputCommand::Drop,
                            VirtualKeyCode::Escape => InputCommand::Escape,
                            _ => InputCommand::None,
                        },
                    },
                    GameMode::VillageSim => match input.virtual_keycode {
                        None => InputCommand::None,
                        Some(key) => match key {
                            VirtualKeyCode::Space => InputCommand::Pause,
                            _ => InputCommand::None,
                        }
                    },
                    _ => InputCommand::None
                };
            }
    
            return cmd;
        }
    }

    InputCommand::None
}

pub fn handle_input(event: WindowEvent, game: &mut Game) -> GameState {
    let player_id = game.world_sim.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;

    let command = map_keys(event, game);

    return command.execute(game, Some(player_id));
}
