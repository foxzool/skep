use crate::{
    entity::{MqttAttributesMixin, MqttDiscoveryUpdateMixin, MqttEntity, MqttEntityDeviceInfo},
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
    constants::EntityCategory,
    helper::{
        device_registry::DeviceInfo,
        entity::{EntityDescription, SkepEntity},
    },
    typing::{ConfigType, SetupConfigEntry, StateType},
};
use skep_sensor::ENTITY_ID_FORMAT;

use bevy_ecs::{prelude::Commands, world::CommandQueue};
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
            entity_id: None,
            entity_description: Default::default(),
            assumed_state: false,
            attribution: None,
            available: false,
            capability_attributes: None,
            device_class: None,
            device_info: None,
            entity_category: None,
            has_entity_name: false,
            entity_picture: None,
            entity_registry_enabled_default: false,
            entity_registry_visible_default: false,
            extra_state_attributes: Default::default(),
            force_update: false,
            icon: None,
            name: None,
            should_poll: false,
            state: None,
            supported_features: None,
            translation_key: None,
            translation_placeholders: Default::default(),
            unique_id: None,
            unit_of_measurement: None,
            config: Default::default(),
            default_name: Some(DEFAULT_NAME.to_string()),
            discovery: None,
            subscriptions: Default::default(),
            entity_id_format: ENTITY_ID_FORMAT.to_string(),
            last_rest: None,
            extra_blocked: MQTT_SENSOR_ATTRIBUTES_BLOCKED.to_owned(),

            expiration_trigger: None,
            expire_after: None,
            expired: None,
        }
    }
}

const DEFAULT_NAME: &str = "MQTT Sensor";
const DEFAULT_FORCE_UPDATE: bool = false;

impl MqttAttributesMixin for MqttSensorComponent {
    fn init(&mut self, config: ConfigType) {
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
    fn set_entity_category(&mut self, entity_category: Option<EntityCategory>) {
        self.entity_category = entity_category;
    }

    fn attr_has_entity_name(&self) -> Option<bool> {
        Some(true)
    }

    fn set_entity_registry_enabled_default(&mut self, entity_registry_enabled_default: bool) {
        self.entity_registry_visible_default = entity_registry_enabled_default;
    }

    fn attr_force_update(&self) -> Option<bool> {
        Some(false)
    }

    fn attr_entity_description(&self) -> Option<EntityDescription> {
        todo!()
    }

    fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
    }

    fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    fn should_poll(&self) -> bool {
        false
    }

    fn set_unique_id(&mut self, unique_id: Option<String>) {
        self.unique_id = unique_id;
    }
}

impl MqttDiscoveryUpdateMixin for MqttSensorComponent {
    fn init(discovery_data: Option<DiscoveryInfoType>) -> Self {
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
    fn init(specifications: Option<HashMap<String, Value>>, config_entry: ConfigEntry) -> Self {
        todo!()
    }
}

impl MqttEntity for MqttSensorComponent {
    fn default_name(&self) -> Option<String> {
        self.default_name.clone()
    }

    fn entity_id_format(&self) -> &str {
        self.entity_id_format.as_str()
    }

    fn config(&self) -> &ConfigType {
        &self.config
    }

    fn set_config(&mut self, config: ConfigType) {
        self.config = config;
    }

    fn set_discovery(&mut self, discovery_data: Option<DiscoveryInfoType>) {
        self.discovery = discovery_data;
    }
}

impl MqttSensorComponent {
    pub fn new(
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> Self {
        let mut sensor = MqttSensorComponent::default();
        sensor.init_mqtt_entity(&config, &config_entry, discovery_data.clone());

        sensor
    }
}

const DOMAIN: &str = "sensor";

fn on_setup_entry(trigger: Trigger<SetupConfigEntry>, mut commands: Commands) {
    if trigger.event().component == DOMAIN {
        let config = ConfigType::default();
        let config_entry = ConfigEntry::default();
        let sensor = MqttSensorComponent::new(config, config_entry, None);
        commands.spawn(sensor);
    }
}
