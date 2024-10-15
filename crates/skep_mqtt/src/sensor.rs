use crate::{
    entity::{MqttAttributesMixin, MqttDiscoveryUpdateMixin, MqttEntity, MqttEntityDeviceInfo},
    subscription::EntitySubscription,
    DiscoveryInfoType, SkepMqttPlatform,
};
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;
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
    CallbackType, SkepResource,
};
use skep_sensor::{SensorDeviceClass, ENTITY_ID_FORMAT};

use crate::{
    binary_sensor::MqttBinarySensorConfiguration,
    discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload, MQTTSupportComponent},
    subscription::MQTTStateSubscription,
};
use bevy_ecs::{
    prelude::{Added, Bundle, Commands, In, ResMut, System},
    system::Query,
    world::CommandQueue,
};
use bevy_log::debug;
use bevy_reflect::Reflect;
use bytes::Bytes;
use lazy_static::lazy_static;
use log::warn;
use serde::{Deserialize, Serialize};
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
        app.add_systems(Update, on_mqtt_platform_added)
            .add_systems(Update, create_or_update_discovery_payload)
            .observe(on_setup_entry);
    }
}

#[derive(Bundle, Debug, Default)]
struct MqttSensorBundle {
    sensor: MqttSensorComponent,
}

impl MqttSensorBundle {
    pub fn new(
        skep_res: ResMut<SkepResource>,
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> Self {
        let sensor = MqttSensorComponent::new(skep_res, config, config_entry, discovery_data);
        MqttSensorBundle { sensor }
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
    attributes_sub_state: HashMap<String, EntitySubscription>,
    attributes_config: Option<ConfigType>,

    available_latest: bool,
    avail_topics: HashMap<String, HashMap<String, Value>>,
    avail_config: Option<ConfigType>,

    discovery_data: Option<DiscoveryInfoType>,
    discovery_update: Option<Box<dyn System<In = In<MQTTDiscoveryPayload>, Out = ()>>>,
    remove_discovery_updated: Option<Box<dyn System<In = In<()>, Out = ()>>>,
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
            attributes_sub_state: Default::default(),
            attributes_config: None,
            available_latest: false,
            avail_topics: Default::default(),
            avail_config: None,
            discovery_data: None,
            discovery_update: None,
            remove_discovery_updated: None,
        }
    }
}

const DEFAULT_NAME: &str = "MQTT Sensor";
const DEFAULT_FORCE_UPDATE: bool = false;

impl MqttAttributesMixin for MqttSensorComponent {
    fn init_attributes(&mut self, config: ConfigType) {
        self.set_attributes_config(config);
    }

    fn attributes_sub_state(&self) -> &HashMap<String, EntitySubscription> {
        todo!()
    }

    fn set_attributes_sub_state(&mut self, sub_state: HashMap<String, EntitySubscription>) {
        todo!()
    }

    fn attributes_config(&self) -> &ConfigType {
        todo!()
    }

    fn set_attributes_config(&mut self, config: ConfigType) {
        self.attributes_config = Some(config);
    }
}

impl SkepEntity for MqttSensorComponent {
    fn set_entity_category(&mut self, entity_category: Option<EntityCategory>) {
        self.entity_category = entity_category;
    }

    fn set_entity_id(&mut self, entity_id: Option<String>) {
        self.entity_id = entity_id;
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
    fn set_discovery_data(&mut self, discovery_data: Option<DiscoveryInfoType>) {
        self.discovery_data = discovery_data;
    }

    fn set_discovery_update(
        &mut self,
        discovery_update: Option<Box<dyn System<In = In<MQTTDiscoveryPayload>, Out = ()>>>,
    ) {
        self.discovery_update = discovery_update;
    }

    fn set_remove_discovery_update(&mut self, callback_type: Option<CallbackType>) {
        self.remove_discovery_updated = callback_type;
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
        skep_res: ResMut<SkepResource>,
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> Self {
        let mut sensor = MqttSensorComponent::default();
        sensor.init_mqtt_entity(skep_res, &config, &config_entry, discovery_data.clone());
        sensor.init_attributes(config.clone());

        sensor
    }
}

const DOMAIN: &str = "sensor";

fn on_setup_entry(
    trigger: Trigger<SetupConfigEntry>,
    mut commands: Commands,
    mut skep_res: ResMut<SkepResource>,
) {
    if trigger.event().component == DOMAIN {
        let config = trigger.event().payload.as_object().unwrap().clone();
        let config_entry = ConfigEntry::default();
        let sensor = MqttSensorComponent::new(skep_res, config, config_entry, None);
        commands.spawn(sensor);
    }
}

#[derive(Component, Reflect, Clone)]
pub struct MqttSensorMarker;

fn on_mqtt_platform_added(
    mut commands: Commands,
    mut q_platform: Query<&mut SkepMqttPlatform, (Added<SkepMqttPlatform>)>,
) {
    for mut platform in q_platform.iter_mut() {
        // let ob = commands.spawn_empty().observe(on_setup_entry).id();
        platform.platforms_loaded.insert(DOMAIN.to_string());
    }
}

fn create_or_update_discovery_payload(
    mut commands: Commands,
    q_discovery: Query<
        (Entity, &MQTTDiscoveryHash, &MQTTDiscoveryPayload),
        Added<MQTTDiscoveryPayload>,
    >,
) {
    for (entity, hash, payload) in q_discovery.iter() {
        if hash.component == DOMAIN {
            if let Ok(mqtt_sensor_configuration) = serde_json::from_value::<MqttSensorConfiguration>(
                Value::from(payload.payload.clone()),
            ) {
                // debug!("mqtt_sensor_configuration {:#?}", mqtt_sensor_configuration);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MqttSensorConfiguration {
    pub availability_topic: Option<String>,
    pub device: Option<DeviceInfo>,
    pub device_class: Option<SensorDeviceClass>,
    /// Flag which defines if the entity should be enabled when first added.
    pub enabled_by_default: Option<bool>,
    /// The category of the entity. When set, the entity category must be diagnostic for sensors.
    pub entity_category: Option<EntityCategory>,
    /// If set, it defines the number of seconds after the sensor’s state expires, if it’s not
    /// updated. After expiry, the sensor’s state becomes unavailable. Default the sensors state
    /// never expires.
    pub expire_after: Option<i32>,
    pub force_update: Option<bool>,
    pub icon: Option<String>,
    pub json_attributes_template: Option<String>,
    pub json_attributes_topic: Option<String>,
    pub last_reset_value_template: Option<String>,
    pub name: Option<String>,
    pub object_id: Option<String>,
    pub options: Option<Value>,
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
    pub suggested_display_precision: Option<i32>,
    pub state_topic: String,
    pub unique_id: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub value_template: Option<String>,
}

#[test]
fn test_mqtt_configuration() {
    let json = r#"
    {
    "availability_topic": "watermeter/connection",
    "device":  {
        "configuration_url": "http://192.168.1.51",
        "identifiers": [
            "watermeter"
        ],
        "manufacturer": "AI on the Edge Device",
        "model": "Meter Digitizer",
        "name": "watermeter",
        "sw_version": "v15.7.0"
    },
    "device_class": "signal_strength",
    "entity_category": "diagnostic",
    "icon": "mdi:wifi",
    "name": "Wi-Fi RSSI",
    "object_id": "watermeter_wifiRSSI",
    "payload_available": "connected",
    "payload_not_available": "connection lost",
    "state_topic": "watermeter/wifiRSSI",
    "unique_id": "watermeter-wifiRSSI",
    "unit_of_measurement": "dBm"
}
    "#;
    let sensor: MqttSensorConfiguration = serde_json::from_str(json).unwrap();

    println!("{:#?}", sensor);
}
