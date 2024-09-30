use crate::{
    entity::{MqttAttributesMixin, MqttDiscoveryUpdateMixin, MqttEntity, MqttEntityDeviceInfo}
    ,
    subscription::EntitySubscription,
    DiscoveryInfoType,
};
use bevy_app::{App, Plugin};
use bevy_ecs::{component::Component, observer::Trigger};
use bevy_utils::HashMap;
use chrono::{DateTime, Utc};
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{EntityCategory, CONF_UNIQUE_ID},
    helper::{
        device_registry::DeviceInfo,
        entity::{EntityDescription, SkepEntity},
    },
    typing::{ConfigType, SetupConfigEntry, StateType},
};
use skep_sensor::ENTITY_ID_FORMAT;

use bevy_ecs::world::CommandQueue;
use bytes::Bytes;
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    static ref MQTT_SENSOR_ATTRIBUTES_BLOCKED: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert(skep_sensor::ATTR_LAST_RESET);
        set.insert(skep_sensor::ATTR_STATE_CLASS);
        set
    };
}

pub struct MqttSensorPlugin;

impl Plugin for MqttSensorPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_setup_entry);
    }
}

#[derive(Debug, Component)]
pub struct MqttSensorComponent {
    entity_id: Option<String>,
    entity_description: EntityDescription,
    assumed_state: bool,
    attribution: Option<String>,
    available: bool,
    capability_attributes: Option<HashMap<String, Value>>,
    device_class: Option<String>,
    device_info: Option<DeviceInfo>,
    entity_category: Option<EntityCategory>,
    has_entity_name: bool,
    entity_picture: Option<String>,
    entity_registry_enabled_default: bool,
    entity_registry_visible_default: bool,
    extra_state_attributes: HashMap<String, Value>,
    force_update: bool,
    icon: Option<String>,
    name: Option<String>,
    should_poll: bool,
    state: StateType,
    supported_features: Option<i32>,
    translation_key: Option<String>,
    translation_placeholders: HashMap<String, String>,
    unique_id: Option<String>,
    unit_of_measurement: Option<String>,

    config: ConfigType,
    default_name: Option<String>,
    discovery: Option<DiscoveryInfoType>,
    subscriptions: HashMap<String, HashMap<String, Value>>,
    entity_id_format: String,
    last_rest: Option<DateTime<Utc>>,
    extra_blocked: HashSet<&'static str>,
    expiration_trigger: Option<CommandQueue>,
    expire_after: Option<i32>,
    expired: Option<bool>,
    // template: Option<Arc<Mutex<dyn Fn(ReceivePayloadType, PayloadSentinel) ->
    // ReceivePayloadType>>>, last_reset_template: Option<Box<dyn Fn(ReceivePayloadType) ->
    // ReceivePayloadType>>,
}

type ReceivePayloadType = Bytes;

impl Default for MqttSensorComponent {
    fn default() -> Self {
        MqttSensorComponent {
            default_name: Some(DEFAULT_NAME.to_string()),
            entity_id_format: ENTITY_ID_FORMAT.to_string(),
            extra_blocked: MQTT_SENSOR_ATTRIBUTES_BLOCKED.to_owned(),
            ..Default::default()
        }
    }
}

const DEFAULT_NAME: &str = "MQTT Sensor";
const DEFAULT_FORCE_UPDATE: bool = false;

impl MqttAttributesMixin for MqttSensorComponent {
    fn new(&mut self, config: ConfigType) {
        todo!()
    }

    fn attributes_sub_state(&self) -> &HashMap<String, EntitySubscription> {
        todo!()
    }

    fn attributes_config(&self) -> &ConfigType {
        todo!()
    }
}

impl SkepEntity for MqttSensorComponent {
    fn attr_has_entity_name(&self) -> Option<bool> {
        Some(true)
    }

    fn attr_force_update(&self) -> Option<bool> {
        Some(false)
    }

    fn attr_entity_description(&self) -> Option<EntityDescription> {
        todo!()
    }

    fn should_poll(&self) -> bool {
        false
    }
}

impl MqttDiscoveryUpdateMixin for MqttSensorComponent {
    fn new(discovery_data: Option<DiscoveryInfoType>) -> Self {
        todo!()
    }

    fn get_device_specifications(&self) -> Option<&HashMap<String, Value>> {
        todo!()
    }

    fn get_config_entry(&self) -> &ConfigEntry {
        todo!()
    }
}

impl MqttEntityDeviceInfo for MqttSensorComponent {
    fn new(specifications: Option<HashMap<String, Value>>, config_entry: ConfigEntry) -> Self {
        todo!()
    }
}

impl MqttEntity for MqttSensorComponent {
    type DefaultName = Option<String>;
    type EntityIdFormat = String;

    fn new(
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> Self {
        let mut entity = MqttSensorComponent::default();
        entity.config = config.clone();
        entity.unique_id = config
            .get(CONF_UNIQUE_ID)
            .map(|v| v.as_str().unwrap().to_string());
        entity.discovery = discovery_data.clone();

        entity
    }
}

const DOMAIN: &str = "sensor";

fn on_setup_entry(trigger: Trigger<SetupConfigEntry>) {
    if trigger.event().component == DOMAIN {
        println!("Setup sensor");
    }
}
