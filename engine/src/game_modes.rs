#[derive(Copy, Clone, PartialEq)]
pub struct GameSettings {
    pub mode: GameMode,
    pub mapsize: (usize, usize),
    pub follow_player: bool,
    pub use_player_los: bool,
    pub show_player: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameMode {
    RL, // trad roguelike, basically bracketlib tutorial in caves
    VillageSim,
    OrcHalls, // Orcs spawn in groups, for testing group tactics
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
            mapsize: (160, 80),
            follow_player: true,
            use_player_los: true,
            show_player: true,
        },
        GameMode::OrcHalls => GameSettings {
            mode,
            mapsize: (80, 40),
            follow_player: true,
            use_player_los: false,
            show_player: true,
        },
    }
}
