use bevy_utils::HashMap;
use serde_json::Value;
use skep_core::{device::DeviceInfo, entity::SkipEntity};

#[derive(Debug)]
pub struct MqttEntity {
    device_specifications: Option<HashMap<String, Value>>,
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

impl MqttEntity {}
