use crate::{
    device::DeviceEntry, integration::Integration, loader::load_config_toml, platform::Platform,
};
use bevy_app::{App, Plugin, Startup};

pub mod config_entry;
pub mod constants;
pub mod device;
pub mod entity;
pub mod integration;
pub mod loader;
pub mod platform;
pub mod typing;

pub struct SkepCorePlugin;

impl Plugin for SkepCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((entity::SkepEntityPlugin, device::SkepDevicePlugin))
            .register_type::<Integration>()
            .register_type::<Platform>()
            .register_type::<DeviceEntry>()
            // .register_type::<DeviceEntry>()
            .add_systems(Startup, load_config_toml);
    }
}
