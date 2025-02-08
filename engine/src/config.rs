use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::world::map::XY;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Settings {
    mode: String,
    mapsize: [i32; 2],  
    follow_player: bool,
    use_player_los: bool,
    show_player: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsList {
    settings: Vec<Settings>,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    pub mode: GameMode,
    pub mapsize: XY,
    pub follow_player: bool,
    pub use_player_los: bool,
    pub show_player: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameMode {
    TestMode,
    RL,
    VillageSim,
    OrcHalls, 
    MapDemo,
    OrcArena,
}

impl From<Settings> for GameSettings {
    fn from(settings: Settings) -> Self {
        GameSettings {
            mode: match settings.mode.as_str() {
                "roguelike" => GameMode::RL,
                "village_sim" => GameMode::VillageSim,
                "map_demo" => GameMode::MapDemo,
                "test_mode" => GameMode::TestMode,
                "orc_halls" => GameMode::OrcHalls,
                "orc_arena" => GameMode::OrcArena,
                _ => GameMode::RL, // Default value
            },
            mapsize: settings.mapsize.into(),
            follow_player: settings.follow_player,
            use_player_los: settings.use_player_los,
            show_player: settings.show_player,
        }
    }
}

// keep this for generating new config files?
// impl Settings {
//     fn new() -> Result<Self, ConfigError> {
//         let builder = Config::builder()
//             // Base configuration
//             .add_source(File::new("config/base", FileFormat::Yaml))
            
//             // Environment-specific overrides
//             .add_source(File::new(&format!("config/{}", "env"), FileFormat::Yaml).required(false))
            
//             // Environment variables take highest priority
//             .add_source(config::Environment::with_prefix("MYAPP"))
            
//             .build()?;
    
//         builder.try_deserialize()
//     }
// }

// keep this to copy method for determining environment and having multiple priorities of config
// pub fn get_config(mode: GameMode) -> Result<GameSettings, Box<dyn std::error::Error>> {
//     // Determine environment
//     let env = env::var("RUN_ENV").unwrap_or_else(|_| "roguelike".to_string());
    
//     let settings = Config::builder()
//         // Base configuration
//         .add_source(File::new("config/base", FileFormat::Yaml))
        
//         // Environment-specific configuration
//         .add_source(File::new(&format!("config/{}", env), FileFormat::Yaml).required(false))
        
//         // Environment variables
//         .add_source(config::Environment::with_prefix("MYAPP"))
        
//         .build()?
//         .try_deserialize::<Settings>()?;

//     Ok(GameSettings {
//         mode,
//         mapsize: settings.mapsize.into(),
//         follow_player: settings.follow_player,
//         use_player_los: settings.use_player_los,
//         show_player: settings.show_player,
//     })
// }

pub fn get_config(mode: GameMode) -> Result<GameSettings, Box<dyn std::error::Error>> {
    let settings = Config::builder()
        .add_source(File::new(&format!("config/{}", "game_modes"), FileFormat::Yaml).required(true))
        .build()?
        .try_deserialize::<SettingsList>()?;

    let env = match mode {
        GameMode::RL => "roguelike",
        GameMode::VillageSim => "village_sim",
        GameMode::MapDemo => "map_demo",
        GameMode::TestMode => "test_mode",
        GameMode::OrcHalls => "orc_halls",
        GameMode::OrcArena => "orc_arena",
    };

    for s in settings.settings.iter() {
        if s.mode == env {
            return Ok(s.clone().into());
        }
    }

    Err("Settings not found".into())

    // dbg!("settings not found, using first setting");

    // let s = GameSettings::from(settings.settings[0].clone());
    // Ok(s)
}
