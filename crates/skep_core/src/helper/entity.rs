use crate::{
    constants::{EntityCategory, STATE_UNKNOWN},
    helper::device_registry::DeviceInfo,
    typing::StateType,
};
use bevy_ecs::component::Component;
use bevy_utils::HashMap;
use either::Either;
use serde_json::Value;

#[allow(dead_code)]
#[derive(Debug, Clone, Component)]
pub struct SkepEntityComponent {
    pub entity_id: Option<String>,
    entity_description: EntityDescription,
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

impl Default for SkepEntityComponent {
    fn default() -> Self {
        SkepEntityComponent {
            entity_registry_enabled_default: true,
            should_poll: true,
            ..Default::default()
        }
    }
}

impl SkepEntityComponent {}

pub trait SkepEntity {
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

    fn set_entity_category(&mut self, entity_category: Option<EntityCategory>);

    fn has_entity_name(&self) -> bool {
        if let Some(has_entity_name) = self.attr_has_entity_name() {
            return has_entity_name;
        }
        if let Some(entity_description) = self.attr_entity_description() {
            return entity_description.has_entity_name;
        }
        false
    }

    fn entity_id(&self) -> Option<String> {
        None
    }

    fn set_entity_id(&mut self, entity_id: Option<String>);

    fn attr_has_entity_name(&self) -> Option<bool>;

    fn entity_picture(&self) -> Option<String> {
        None
    }

    fn entity_registry_enabled_default(&self) -> bool {
        true
    }

    fn set_entity_registry_enabled_default(&mut self, entity_registry_enabled_default: bool);

    fn entity_registry_visible_default(&self) -> bool {
        true
    }

    fn extra_state_attributes(&self) -> Option<HashMap<String, Value>> {
        None
    }

    fn force_update(&self) -> bool {
        if let Some(force_update) = self.attr_force_update() {
            return force_update;
        }
        if let Some(entity_description) = self.attr_entity_description() {
            return entity_description.force_update;
        }
        false
    }

    fn attr_force_update(&self) -> Option<bool>;

    fn attr_entity_description(&self) -> Option<EntityDescription>;

    fn icon(&self) -> Option<String> {
        None
    }

    fn set_icon(&mut self, icon: Option<String>);

    fn name(&self) -> Option<String> {
        None
    }

    fn set_name(&mut self, name: Option<String>);

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

    fn set_unique_id(&mut self, unique_id: Option<String>);

    fn unit_of_measurement(&self) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Default)]
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
    pub translation_placeholders: Option<std::collections::HashMap<String, String>>,
    pub unit_of_measurement: Option<String>,
}
