use engine::{GameMode, GameSettings};

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
            mapsize: (80, 40),
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
