use crate::{
    constants::{CONF_ENABLED_BY_DEFAULT, CONF_OBJECT_ID},
    subscription::EntitySubscription,
    DiscoveryInfoType,
};
use bevy_ecs::prelude::ResMut;
use bevy_utils::HashMap;
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{
        EntityCategory, CONF_DEVICE, CONF_ENTITY_CATEGORY, CONF_ICON, CONF_NAME, CONF_UNIQUE_ID,
    },
    helper::entity::SkepEntity,
    typing::ConfigType,
    SkepResource,
};
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct MqttEntity {
    device_specifications: Option<HashMap<String, Value>>,
    config: ConfigType,
    sub_state: HashMap<String, EntitySubscription>,
    discovery: bool,
    subscriptions: HashMap<String, HashMap<String, Value>>,
    default_name: Option<String>,
    entity_id_format: String,
}

impl MqttEntity {
    pub fn new(
        skep_res: &ResMut<SkepResource>,
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> anyhow::Result<(SkepEntity, MqttEntity)> {
        let mut skep_entity = SkepEntity::default();
        let mut mqtt_entity = MqttEntity::default();
        mqtt_entity.config = config.clone();
        skep_entity.unique_id = config
            .get(CONF_UNIQUE_ID)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        mqtt_entity.discovery = discovery_data.is_some();

        mqtt_entity.setup_common_attributes_from_config(&mut skep_entity, config);
        mqtt_entity.init_entity_id();

        Ok((skep_entity, mqtt_entity))
    }

    fn setup_common_attributes_from_config(
        &mut self,
        skep_entity: &mut SkepEntity,
        config: ConfigType,
    ) {
        skep_entity.entity_category = config.get(CONF_ENTITY_CATEGORY).and_then(|v| {
            v.as_str().and_then(|s| {
                EntityCategory::from_str(s)
                    .map_err(|_| anyhow::anyhow!("Invalid entity category: {}", s))
                    .ok()
            })
        });
        skep_entity.entity_registry_enabled_default = config
            .get(CONF_ENABLED_BY_DEFAULT)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        skep_entity.icon = config
            .get(CONF_ICON)
            .and_then(|v| v.as_str().map(|s| s.to_string()));
    }

    fn set_entity_name(&mut self, skep_entity: &mut SkepEntity, config: ConfigType) {
        match config.get(CONF_NAME) {
            Some(entity_name) => {
                skep_entity.name = entity_name.as_str().map(|s| s.to_string());
            }
            None => {
                if !self.default_to_device_class_name() {
                    skep_entity.name = self.default_name.clone();
                } else {
                    skep_entity.name = None;
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

    fn default_to_device_class_name(&self) -> bool {
        false
    }

    fn init_entity_id(&mut self) {}
}

fn init_entity_id_from_config(
    skep_res: &ResMut<SkepResource>,
    skep_entity: &mut SkepEntity,
    config: &ConfigType,
    entity_id_format: &str,
) {
    if let Some(object_id) = config.get(CONF_OBJECT_ID) {
        // skep_entity.entity_id = ;
    }
}
