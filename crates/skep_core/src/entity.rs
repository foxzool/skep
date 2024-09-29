use bevy_app::prelude::*;

pub(crate) struct SkepEntityPlugin;

impl Plugin for SkepEntityPlugin {
    fn build(&self, _app: &mut App) {}
}

use crate::{
    constants::{DEVICE_DEFAULT_NAME, STATE_UNKNOWN},
    device::DeviceInfo,
    typing::StateType,
};
use dyn_fmt::AsStrFormatExt;
use either::Either;
use serde_json::Value;
use slugify::slugify;
use std::collections::HashMap;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, Clone)]
#[strum(serialize_all = "snake_case")]
#[strum(ascii_case_insensitive)]
pub enum EntityCategory {
    Config,
    Diagnostic,
}

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

pub trait SkipEntity {
    fn assumed_state(&self) -> bool {
        true
    }

    fn attribution(&self) -> Option<String> {
        None
    }

    fn available(&self) -> bool {
        true
    }

    fn capability_attributes(&self) -> Option<HashMap<String, Value>> {
        None
    }

    fn device_class(&self) -> Option<String> {
        None
    }

    fn device_info(&self) -> Option<DeviceInfo> {
        None
    }

    fn entity_category(&self) -> Option<EntityCategory> {
        None
    }

    fn has_entity_name(&self) -> bool {
        false
    }

    fn entity_picture(&self) -> Option<String> {
        None
    }

    fn entity_registry_enabled_default(&self) -> bool {
        true
    }

    fn entity_registry_visible_default(&self) -> bool {
        true
    }

    fn extra_state_attributes(&self) -> Option<HashMap<String, Value>> {
        None
    }

    fn force_update(&self) -> bool {
        false
    }

    fn icon(&self) -> Option<String> {
        None
    }

    fn name(&self) -> Option<String> {
        None
    }

    fn should_poll(&self) -> bool {
        true
    }

    fn state(&self) -> StateType {
        Some(Either::Left(STATE_UNKNOWN.to_string()))
    }

    fn supported_features(&self) -> Option<i32> {
        None
    }

    fn translation_key(&self) -> Option<String> {
        None
    }

    fn translation_placeholders(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn unique_id(&self) -> Option<String> {
        None
    }

    fn unit_of_measurement(&self) -> Option<String> {
        None
    }

    fn default_to_device_class_name(&self) -> bool {
        false
    }
}

pub fn generate_entity_id(
    entity_id_format: &str,
    name: Option<&str>,
    current_ids: Option<&[String]>,
    // id_query: Query<impl SkipEntity + Component>,
) -> Result<String, &'static str> {
    let name = name.unwrap_or(DEVICE_DEFAULT_NAME).to_lowercase();
    let preferred_string = entity_id_format.format(&[slugify!(&name, separator = "_")]);

    // if let Some(current_ids) = current_ids {
    //     return Ok(ensure_unique_string(&preferred_string, current_ids));
    // }

    let mut test_string = preferred_string.clone();
    let mut tries = 1;
    // while !hass.states.async_available(&test_string) {
    //     tries += 1;
    //     test_string = format!("{}_{}", preferred_string, tries);
    // }

    Ok(test_string)
}

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
