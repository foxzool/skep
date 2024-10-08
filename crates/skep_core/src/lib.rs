use crate::{
    constants::DEVICE_DEFAULT_NAME, device::SkepDevicePlugin, domain::Domain,
    entity::SkepEntityPlugin, helper::event::SkepCoreEventPlugin, integration::Integration,
    loader::load_config_toml, platform::Platform,
};
use bevy_app::{App, Plugin, Startup};
use bevy_ecs::system::Resource;
use bevy_reflect::Reflect;
use bevy_utils::HashSet;
use dyn_fmt::AsStrFormatExt;
use slugify::slugify;

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

impl SkepResource {
    pub fn generate_entity_id(
        &mut self,
        entity_id_format: &str,
        name: Option<&str>,
        current_ids: Option<Vec<String>>,
    ) -> anyhow::Result<String> {
        let name = name.unwrap_or(DEVICE_DEFAULT_NAME).to_lowercase();
        let preferred_string = entity_id_format.format(&[slugify!(&name, separator = "_")]);

        if let Some(current_ids) = current_ids {
            return Ok(ensure_unique_string(
                &preferred_string,
                current_ids.iter().map(String::to_string),
            ));
        }

        let mut test_string = preferred_string.clone();
        let mut tries = 1;
        while !self.entity_ids.contains(&test_string) {
            tries += 1;
            test_string = format!("{}_{}", preferred_string, tries);
        }

        Ok(test_string)
    }
}

fn ensure_unique_string(
    preferred_string: &str,
    current_strings: impl IntoIterator<Item = String>,
) -> String {
    let mut test_string = preferred_string.to_string();
    let current_strings_set: std::collections::HashSet<String> =
        current_strings.into_iter().collect();

    let mut tries = 1;

    while current_strings_set.contains(&test_string) {
        tries += 1;
        test_string = format!("{}_{}", preferred_string, tries);
    }

    test_string
}
