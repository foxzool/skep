use crate::{
    constants::Platform, integration::Integration, loader::load_config_toml,
    typing::SetupConfigEvent,
};
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
            .register_type::<Integration>()
            .register_type::<Platform>()
            .add_event::<SetupConfigEvent>()
            .add_systems(Startup, load_config_toml);
    }
}
