use bevy_app::prelude::*;

pub(crate) struct SkepEntityPlugin;

impl Plugin for SkepEntityPlugin {
    fn build(&self, _app: &mut App) {}
}

use crate::constants::DEVICE_DEFAULT_NAME;
use dyn_fmt::AsStrFormatExt;
use slugify::slugify;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EntityDescription {
    pub key: String,
    pub device_class: Option<String>,
    pub entity_category: Option<String>,
    pub entity_registry_enabled_default: bool,
    pub entity_registry_visible_default: bool,
    pub force_update: bool,
    pub icon: Option<String>,
    pub has_entity_name: bool,
    pub name: Option<String>,
    pub translation_key: Option<String>,
    pub translation_placeholders: Option<HashMap<String, String>>,
    pub unit_of_measurement: Option<String>,
}

pub fn generate_entity_id(
    entity_id_format: &str,
    name: Option<&str>,
    current_ids: Option<&[String]>,
    skep_res: &ResMut<SkepResource>,
) -> Result<String, &'static str> {
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
    while !skep_res.entity_ids.contains(&test_string) {
        tries += 1;
        test_string = format!("{}_{}", preferred_string, tries);
    }

    Ok(test_string)
}

use crate::SkepResource;
use bevy_ecs::prelude::ResMut;
use std::collections::HashSet;

fn ensure_unique_string(
    preferred_string: &str,
    current_strings: impl IntoIterator<Item = String>,
) -> String {
    let mut test_string = preferred_string.to_string();
    let current_strings_set: HashSet<String> = current_strings.into_iter().collect();

    let mut tries = 1;

    while current_strings_set.contains(&test_string) {
        tries += 1;
        test_string = format!("{}_{}", preferred_string, tries);
    }

    test_string
}
