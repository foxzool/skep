use crate::{
    device::SkepDevicePlugin, domain::Domain, entity::SkepEntityPlugin,
    helper::event::SkepCoreEventPlugin, integration::Integration, loader::load_config_toml,
    platform::Platform,
};
use bevy_app::{App, Plugin, Startup};
use bevy_ecs::system::Resource;
use bevy_reflect::Reflect;
use bevy_utils::HashSet;

pub mod config_entry;
pub mod constants;
pub mod device;
pub mod domain;
pub mod entity;
pub mod helper;
pub mod integration;
pub mod loader;
pub mod platform;
pub mod states;
pub mod typing;

pub struct SkepCorePlugin;

impl Plugin for SkepCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SkepEntityPlugin, SkepDevicePlugin, SkepCoreEventPlugin))
            .register_type::<Integration>()
            .register_type::<Platform>()
            .register_type::<Domain>()
            .register_type::<SkepResource>()
            .init_resource::<SkepResource>()
            // .register_type::<DeviceEntry>()
            .add_systems(Startup, load_config_toml);
    }
}

#[derive(Debug, Resource, Default, Reflect)]
pub struct SkepResource {
    pub entity_ids: HashSet<String>,
}
