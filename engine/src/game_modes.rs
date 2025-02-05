use crate::simulator::map::XY;

#[derive(Copy, Clone, PartialEq)]
pub struct GameSettings {
    pub mode: GameMode,
    pub mapsize: XY,
    pub follow_player: bool,
    pub use_player_los: bool,
    pub show_player: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameMode {
    TestMode,
    RL,
    VillageSim,
    OrcHalls, 
    MapDemo,
    OrcArena,
}

pub fn get_settings(mode: GameMode) -> GameSettings {
    match mode {
        GameMode::VillageSim => GameSettings {
            mode,
            mapsize: (200, 80),
            follow_player: false,
            use_player_los: false,
            show_player: false,
        },
        GameMode::RL => GameSettings {
            mode,
            mapsize: (70, 40),
            follow_player: false, // true, 
            use_player_los: false, // true,
            show_player: true,
        },
        GameMode::OrcHalls => GameSettings {
            mode,
            mapsize: (80, 40),
            follow_player: true,
            use_player_los: false,
            show_player: true,
        },
        GameMode::MapDemo => GameSettings {
            mode,
            mapsize: (160, 80),
            follow_player: false,
            use_player_los: false,
            show_player: false,
        },
        GameMode::OrcArena => GameSettings {
            mode,
            mapsize: (160, 80),
            follow_player: false,
            use_player_los: false,
            show_player: false,
        },
        GameMode::TestMode => GameSettings {
            mode,
            mapsize: (160, 80),
            follow_player: false,
            use_player_los: false,
            show_player: true,
        },
    }
}
