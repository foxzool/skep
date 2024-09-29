use crate::{subscription::EntitySubscription, DiscoveryInfoType};
use bevy_utils::HashMap;
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry, constants::CONF_UNIQUE_ID, device::DeviceInfo, entity::SkipEntity,
    typing::ConfigType,
};

#[derive(Debug, Default)]
pub struct MqttEntity {
    device_specifications: Option<HashMap<String, Value>>,
    config: ConfigType,
    unique_id: Option<String>,
    sub_state: HashMap<String, EntitySubscription>,
    discovery: bool,
    subscriptions: HashMap<String, HashMap<String, Value>>,
}

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
        Ok(mqtt_entity)
    }
}
