use crate::{
    constants::CONF_ENABLED_BY_DEFAULT, subscription::EntitySubscription, DiscoveryInfoType,
};
use bevy_utils::HashMap;
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{CONF_DEVICE, CONF_ENTITY_CATEGORY, CONF_ICON, CONF_NAME, CONF_UNIQUE_ID},
    device::DeviceInfo,
    entity::{EntityCategory, SkipEntity},
    typing::ConfigType,
};
use std::str::FromStr;

impl SkipEntity for MqttEntity {
    fn device_info(&self) -> Option<DeviceInfo> {
        serde_json::from_value(serde_json::json!(self.device_specifications)).ok()
    }

    fn has_entity_name(&self) -> bool {
        true
    }

    fn force_update(&self) -> bool {
        false
    }

    fn should_poll(&self) -> bool {
        false
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn entity_category(&self) -> Option<EntityCategory> {
        self.entity_category.clone()
    }

    fn entity_registry_enabled_default(&self) -> bool {
        self.entity_registry_enabled_default.unwrap_or(true)
    }

    fn icon(&self) -> Option<String> {
        self.icon.clone()
    }
}

#[derive(Debug, Default)]
pub struct MqttEntity {
    device_specifications: Option<HashMap<String, Value>>,
    config: ConfigType,
    unique_id: Option<String>,
    sub_state: HashMap<String, EntitySubscription>,
    discovery: bool,
    subscriptions: HashMap<String, HashMap<String, Value>>,
    entity_category: Option<EntityCategory>,
    entity_registry_enabled_default: Option<bool>,
    icon: Option<String>,
    name: Option<String>,
    default_name: Option<String>,
    entity_id_format: String,
}

impl MqttEntity {
    pub fn new(
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> anyhow::Result<MqttEntity> {
        let mut mqtt_entity = MqttEntity::default();
        mqtt_entity.config = config.clone();
        mqtt_entity.unique_id = config
            .get(CONF_UNIQUE_ID)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        mqtt_entity.discovery = discovery_data.is_some();

        mqtt_entity.setup_common_attributes_from_config(config);

        Ok(mqtt_entity)
    }

    fn setup_common_attributes_from_config(&mut self, config: ConfigType) {
        self.entity_category = config.get(CONF_ENTITY_CATEGORY).and_then(|v| {
            v.as_str().and_then(|s| {
                EntityCategory::from_str(s)
                    .map_err(|_| anyhow::anyhow!("Invalid entity category: {}", s))
                    .ok()
            })
        });
        self.entity_registry_enabled_default = config
            .get(CONF_ENABLED_BY_DEFAULT)
            .and_then(|v| v.as_bool());
        self.icon = config
            .get(CONF_ICON)
            .and_then(|v| v.as_str().map(|s| s.to_string()));
    }

    fn set_entity_name(&mut self, config: ConfigType) {
        match config.get(CONF_NAME) {
            Some(entity_name) => {
                self.name = entity_name.as_str().map(|s| s.to_string());
            }
            None => {
                if !self.default_to_device_class_name() {
                    self.name = self.default_name.clone();
                } else {
                    self.name = None;
                }
            }
        }

        if let Some(device) = config.get(CONF_DEVICE).and_then(|v| v.as_object()) {
            if !device.contains_key(CONF_NAME) {
                log::info!(
                    "MQTT device information always needs to include a name, got {:?}, \
if device information is shared between multiple entities, the device \
name must be included in each entity's device configuration",
                    config
                );
            }
        }
    }
}
