

/*

This code is not in use yet


*/



use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Mode {
    mapsize: [i32; 2],  
    follow_player: bool,
    use_player_los: bool,
    show_player: bool,
}

// mode:
//   mapsize: [i32; 2],
//   follow_player: false,
//   use_player_los: false,
//   show_player: false,

// #[derive(Debug, Serialize, Deserialize)]
// struct DatabaseConfig {
//     url: String,
//     max_connections: usize,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct ServerConfig {
//     host: String,
//     port: u16,
// }

// // keep this for generating new config files?
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

fn get_config() -> Result<(), Box<dyn std::error::Error>> {
    // Determine environment
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "roguelike".to_string());
    
    let settings = Config::builder()
        // Base configuration
        .add_source(File::new("config/base", FileFormat::Yaml))
        
        // Environment-specific configuration
        .add_source(File::new(&format!("config/{}", env), FileFormat::Yaml).required(false))
        
        // Environment variables
        .add_source(config::Environment::with_prefix("MYAPP"))
        
        .build()?
        .try_deserialize::<Mode>()?;

    dbg!(settings);

    Ok(())
}
