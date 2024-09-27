use bevy_ecs::{event::Event, system::Commands};
use log::{debug, error, warn};
use serde_toml_merge::merge;

#[derive(Event)]
pub struct LoadConfig {
    pub config: serde_json::Value,
}

pub fn load_config_toml(mut commands: Commands) {
    match load_config_file() {
        Ok(config) => {
            let config = serde_json::to_value(config).unwrap();
            debug!(
                "Config loaded {}",
                serde_json::to_string_pretty(&config).unwrap()
            );
            commands.trigger(LoadConfig { config });
        }
        Err(e) => error!("Failed to load config: {:?}", e),
    }
}

fn load_config_file() -> anyhow::Result<toml::Value> {
    let default_config_path = "config/default.toml";
    let str = std::fs::read_to_string(default_config_path)?;
    let default_config: toml::Value = toml::from_str(&str)?;

    let local_config_path = "config/local.toml";
    let config = if let Ok(str) = std::fs::read_to_string(local_config_path) {
        let local_config: toml::Value = toml::from_str(&str)?;
        match merge(default_config.clone(), local_config) {
            Ok(new_config) => new_config,
            Err(e) => {
                warn!("Failed to merge config: {:?}", e);
                default_config
            }
        }
    } else {
        default_config
    };

    Ok(config)
}
