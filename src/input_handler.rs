use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::{Game, GameState, screen::menu_config::MainMenuSelection};

pub enum Action {
    None,
    Exit,
}

pub fn handle_input(input: &WinitInputHelper, game: &mut Game) -> Action {
    // Esc : Exit
    if input.key_pressed(VirtualKeyCode::Escape) {
        match game.state {
            GameState::MainMenu { selection } => return Action::Exit,
            _ => game.set_state(GameState::MainMenu { selection: MainMenuSelection::Play }),
        }
    }

    // + : zoom in
    if input.key_pressed_os(VirtualKeyCode::Equals) {
        game.screen.increment_zoom();
    }

    // - : zoom out
    if input.key_pressed_os(VirtualKeyCode::Minus) {
        game.screen.decrement_zoom();
    }

    // R : reset
    if input.key_pressed_os(VirtualKeyCode::R) {
        game.engine.reset_engine(game.engine.settings);
        game.set_state(GameState::ShowMapHistory);
    }

    let movemod = if input.held_shift() {
        10
    } else {
        1
    };

    // Up
    if input.key_pressed_os(VirtualKeyCode::Up) {
        match game.state {
            GameState::Waiting => game.screen.pan_map((0, -1 * movemod)),
            GameState::MainMenu { selection } => game.set_state(GameState::MainMenu { selection: selection.dec() }),
            _ => {},
        }
    }

    // Down
    if input.key_pressed_os(VirtualKeyCode::Down) {
        match game.state {
            GameState::Waiting => game.screen.pan_map((0, 1 * movemod)),
            GameState::MainMenu { selection } => game.set_state( GameState::MainMenu { selection: selection.inc() }),
            _ => {},
        }
    }

    // Left
    if input.key_pressed_os(VirtualKeyCode::Left) {
        match game.state {
            GameState::Waiting => game.screen.pan_map((-1 * movemod, 0)),
            _ => {},
        }
    }

    // Right
    if input.key_pressed_os(VirtualKeyCode::Right) {
        match game.state {
            GameState::Waiting => game.screen.pan_map((1 * movemod, 0)),
            _ => {},
        }
    }

    // Enter
    if input.key_pressed_os(VirtualKeyCode::Return) {
        match game.state {
            GameState::MainMenu { selection } => {
                match selection {
                    MainMenuSelection::Play => game.set_state(GameState::ShowMapHistory),
                    MainMenuSelection::ModeSelect => todo!(),
                    MainMenuSelection::Quit => return Action::Exit,
                }
            },
            _ => {},
        }
    }

    return Action::None;
}
