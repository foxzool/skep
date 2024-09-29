use bevy_app::prelude::*;

pub(crate) struct SkepEntityPlugin;

impl Plugin for SkepEntityPlugin {
    fn build(&self, _app: &mut App) {}
}

use crate::{constants::STATE_UNKNOWN, device::DeviceInfo, typing::StateType};
use either::Either;
use serde_json::Value;
use std::collections::HashMap;

enum EntityCategory {
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
}
