use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

pub struct SkepEntityPlugin;

impl Plugin for SkepEntityPlugin {
    fn build(&self, _app: &mut App) {}
}

use serde_json::{Map, Value};
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

#[derive(Component, Debug)]
pub struct SkepEntity {
    assumed_state: bool,
    attribution: Option<String>,
    available: bool,
    device_class: Option<String>,
    entity_picture: Option<String>,
    extra_state_attributes: Option<Map<String, Value>>,
    has_entity_name: bool,
    name: Option<String>,
    should_pull: bool,
    state: Option<String>,
    supported_features: Option<i32>,
    translation_key: Option<String>,
    translation_placeholders: Option<HashMap<String, String>>,
}
