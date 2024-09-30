use crate::{constants::EntityCategory, helper::device_registry::DeviceInfo, typing::StateType};
use bevy_ecs::component::Component;
use bevy_utils::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Component)]
pub struct SkepEntity {
    pub entity_id: Option<String>,
    assumed_state: bool,
    attribution: Option<String>,
    available: bool,
    capability_attributes: Option<HashMap<String, Value>>,
    device_class: Option<String>,
    device_info: Option<DeviceInfo>,
    pub entity_category: Option<EntityCategory>,
    has_entity_name: bool,
    entity_picture: Option<String>,
    pub entity_registry_enabled_default: bool,
    entity_registry_visible_default: bool,
    extra_state_attributes: HashMap<String, Value>,
    force_update: bool,
    pub icon: Option<String>,
    pub name: Option<String>,
    should_poll: bool,
    state: StateType,
    supported_features: Option<i32>,
    translation_key: Option<String>,
    translation_placeholders: HashMap<String, String>,
    pub unique_id: Option<String>,
    unit_of_measurement: Option<String>,
}

impl Default for SkepEntity {
    fn default() -> Self {
        SkepEntity {
            entity_registry_enabled_default: true,
            should_poll: true,
            ..Default::default()
        }
    }
}

impl SkepEntity {}
