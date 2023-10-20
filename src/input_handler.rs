use std::collections::HashMap;

use engine::{
    components::{Item, PPoint, PlayerID},
    effects::{add_effect, EffectType},
    map::Map,
    utils::dir_to_point, game_modes::{GameMode, get_settings}, player,
};
use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View};
use winit::event::{WindowEvent, VirtualKeyCode};

use crate::{Game, GameState, screen::menu_config::{MainMenuSelection, ModeSelectSelection}};


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

    //debug
    Reset,

    //ui
    ZoomIn,
    ZoomOut,
    Enter
}

impl InputCommand {
    fn execute(&self, game: &mut Game, creator: Option<EntityId>) -> GameState {
        let world = &game.engine.world;

        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;

        // return GameState::AwaitingInput to ignore input, GameState::PlayerTurn to advance engine
        return match self {
            InputCommand::None => GameState::Waiting,
            InputCommand::Move { dir } => {
                let map: UniqueView<'_, Map> = world.borrow::<UniqueView<Map>>().unwrap();

                // hold shift to move by 10 squares at a time
                let movemod = 1;

                let mut dir_targets: HashMap<i32, usize> = HashMap::new();
                dir_targets.insert(1, map.point_idx(dir_to_point(player_pos, 1, movemod)));
                dir_targets.insert(2, map.point_idx(dir_to_point(player_pos, 2, movemod)));
                dir_targets.insert(3, map.point_idx(dir_to_point(player_pos, 3, movemod)));
                dir_targets.insert(4, map.point_idx(dir_to_point(player_pos, 4, movemod)));
                dir_targets.insert(6, map.point_idx(dir_to_point(player_pos, 6, movemod)));
                dir_targets.insert(7, map.point_idx(dir_to_point(player_pos, 7, movemod)));
                dir_targets.insert(8, map.point_idx(dir_to_point(player_pos, 8, movemod)));
                dir_targets.insert(9, map.point_idx(dir_to_point(player_pos, 9, movemod)));

                add_effect(
                    creator,
                    EffectType::MoveOrAttack {
                        tile_idx: dir_targets[dir],
                    },
                );

                GameState::PlayerTurn
            }
            InputCommand::ShowInventory => GameState::ShowInventory,
            InputCommand::Wait => {
                add_effect(creator, EffectType::Wait {}); //todo is this weird on sim mode?
                GameState::PlayerTurn
            }
            InputCommand::Escape => {
                match game.state {
                    GameState::MainMenu { selection: _ } => GameState::Exit,
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

                GameState::PlayerTurn
            }
            InputCommand::Explore => {
                add_effect(creator, EffectType::Explore {});

                GameState::PlayerTurn
            }
            InputCommand::RevealMap => {
                player::reveal_map(&world);

                GameState::PlayerTurn
            }
            InputCommand::Fireball => {
                dbg!("fireball is broken");
                GameState::Waiting
                // GameState::ShowTargeting {
                //     range: 6,
                //     item: world.run(|mut store: AllStoragesViewMut| {
                //         entity_factory::tmp_fireball(&mut store)
                //     }),
                // }
            }
            InputCommand::UseStairs => {
                // if player::try_next_level(&world) {
                //     GameState::NextLevel
                // } else {
                //     GameState::AwaitingInput
                // }
                GameState::Waiting
            }
            InputCommand::Reset => {
                game.engine.reset_engine(game.engine.settings);
                GameState::ShowMapHistory
            },
            InputCommand::ZoomIn => {
                game.screen.increment_zoom();
                GameState::Waiting
            },
            InputCommand::ZoomOut => {
                game.screen.decrement_zoom();
                GameState::Waiting
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
                        match selection {
                            ModeSelectSelection::MapDemo => game.engine.reset_engine(get_settings(GameMode::MapDemo)),
                            ModeSelectSelection::RL => game.engine.reset_engine(get_settings(GameMode::RL)),
                            ModeSelectSelection::VillageSim => game.engine.reset_engine(get_settings(GameMode::VillageSim)),
                        }

                        GameState::ShowMapHistory
                    }
                    _ => GameState::Waiting,
                }
            },
        };
    }
}




//     // Esc : Exit
//     if input.key_pressed(VirtualKeyCode::Escape) {
//         match game.state {
//             GameState::MainMenu { selection: _ } => return Action::Exit,
//             _ => game.set_state(GameState::MainMenu { selection: MainMenuSelection::Play }),
//         }
//     }

//     // + : zoom in
//     if input.key_pressed_os(VirtualKeyCode::Equals) {
//         game.screen.increment_zoom();
//     }

//     // - : zoom out
//     if input.key_pressed_os(VirtualKeyCode::Minus) {
//         game.screen.decrement_zoom();
//     }

//     // R : reset
//     if input.key_pressed_os(VirtualKeyCode::R) {
//     }

//     let movemod = if input.held_shift() {
//         10
//     } else {
//         1
//     };

//     // Up
//     if input.key_pressed_os(VirtualKeyCode::Up) {
//         match game.state {
//             GameState::Waiting => game.screen.pan_map((0, -1 * movemod)),
//             GameState::MainMenu { selection } => game.set_state(GameState::MainMenu { selection: selection.dec() }),
//             GameState::ModeSelect { selection } => game.set_state(GameState::ModeSelect { selection: selection.dec() }),
//             _ => {},
//         }
//     }

//     // Down
//     if input.key_pressed_os(VirtualKeyCode::Down) {
//         match game.state {
//             GameState::Waiting => game.screen.pan_map((0, 1 * movemod)),
//             GameState::MainMenu { selection } => game.set_state( GameState::MainMenu { selection: selection.inc() }),
//             GameState::ModeSelect { selection } => game.set_state( GameState::ModeSelect { selection: selection.inc() }),
//             _ => {},
//         }
//     }

//     // Left
//     if input.key_pressed_os(VirtualKeyCode::Left) {
//         match game.state {
//             GameState::Waiting => game.screen.pan_map((-1 * movemod, 0)),
//             _ => {},
//         }
//     }

//     // Right
//     if input.key_pressed_os(VirtualKeyCode::Right) {
//         match game.state {
//             GameState::Waiting => game.screen.pan_map((1 * movemod, 0)),
//             _ => {},
//         }
//     }

//     // Enter
//     if input.key_pressed_os(VirtualKeyCode::Return) {
//         match game.state {
//             GameState::MainMenu { selection } => {
//                 match selection {
//                     MainMenuSelection::Play => game.set_state(GameState::ShowMapHistory),
//                     MainMenuSelection::ModeSelect => game.set_state(GameState::ModeSelect { selection: ModeSelectSelection::from_repr(0).unwrap() }),
//                     MainMenuSelection::Quit => return Action::Exit,
//                 }
//             },
//             GameState::ModeSelect { selection } => {
//                 match selection {
//                     ModeSelectSelection::MapDemo => game.engine.reset_engine(get_settings(GameMode::MapDemo)),
//                     ModeSelectSelection::RL => game.engine.reset_engine(get_settings(GameMode::RL)),
//                     ModeSelectSelection::VillageSim => game.engine.reset_engine(get_settings(GameMode::VillageSim)),
//                 }

//                 game.set_state(GameState::ShowMapHistory)
//             }
//             _ => {},
//         }
//     }

//     return Action::None;
// }

pub fn map_keys(event: WindowEvent, mode: GameMode) -> InputCommand {
    //universal commands
    if let WindowEvent::KeyboardInput { device_id, input, is_synthetic } = event {
        let mut cmd = match input.virtual_keycode {
            None => InputCommand::None,
            Some(key) => match key {
                VirtualKeyCode::Left => InputCommand::Move { dir: 4 },
                VirtualKeyCode::Right => InputCommand::Move { dir: 6 },
                VirtualKeyCode::Up => InputCommand::Move { dir: 8 },
                VirtualKeyCode::Down => InputCommand::Move { dir: 2 },
                VirtualKeyCode::Y => InputCommand::Move { dir: 7 },
                VirtualKeyCode::U => InputCommand::Move { dir: 9 },
                VirtualKeyCode::N => InputCommand::Move { dir: 3 },
                VirtualKeyCode::B => InputCommand::Move { dir: 1 },
                VirtualKeyCode::F => InputCommand::Fireball,
                VirtualKeyCode::W => InputCommand::Wait,
                VirtualKeyCode::R => InputCommand::Reset,
                VirtualKeyCode::Return => InputCommand::Enter,
                VirtualKeyCode::Escape => InputCommand::Escape,
                VirtualKeyCode::Equals => InputCommand::ZoomIn,
                VirtualKeyCode::Minus => InputCommand::ZoomOut,
                _ => InputCommand::None,
            },
        };

        if cmd == InputCommand::None {
            cmd = match mode {
                GameMode::RL | GameMode::OrcHalls => match input.virtual_keycode {
                    None => InputCommand::None,
                    Some(key) => match key {
                        VirtualKeyCode::G => InputCommand::Get,
                        VirtualKeyCode::X => InputCommand::Explore,
                        VirtualKeyCode::R => InputCommand::RevealMap,
                        VirtualKeyCode::I => InputCommand::ShowInventory,
                        VirtualKeyCode::Escape => InputCommand::Escape,
                        _ => InputCommand::None,
                    },
                },
                _ => InputCommand::None
            };
        }

        return cmd;
    }

    InputCommand::None

    // // mode specific commands
    // return match mode {
    //     GameMode::RL | GameMode::OrcHalls => match input.virtual_keycode {
    //         None => InputCommand::None,
    //         Some(key) => match key {
    //             VirtualKeyCode::G => InputCommand::Get,
    //             VirtualKeyCode::X => InputCommand::Explore,
    //             VirtualKeyCode::R => InputCommand::RevealMap,
    //             VirtualKeyCode::I => InputCommand::ShowInventory,
    //             VirtualKeyCode::Escape => InputCommand::Escape,
    //             _ => InputCommand::None,
    //         },
    //     },
    //     _ => {}
    // };
}

pub fn handle_input(event: WindowEvent, game: &mut Game) -> GameState {
    let settings = game.engine.settings;//world.borrow::<UniqueView<GameSettings>>().unwrap();
    let player_id = game.engine.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;

    let command = map_keys(event, settings.mode);

    return command.execute(game, Some(player_id));
}
