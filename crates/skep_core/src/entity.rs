use bevy_app::prelude::*;

pub(crate) struct SkepEntityPlugin;

impl Plugin for SkepEntityPlugin {
    fn build(&self, _app: &mut App) {}
}

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
