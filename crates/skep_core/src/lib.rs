use crate::loader::load_config_toml;
use bevy_app::{App, Plugin, Startup};

pub mod config_entry;
pub mod constants;
pub mod entity;
pub mod entity_platform;
pub mod integration;
pub mod loader;
pub mod typing;

pub struct SkepCorePlugin;

impl Plugin for SkepCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(entity::SkepEntityPlugin)
            .add_systems(Startup, load_config_toml);
    }
}
