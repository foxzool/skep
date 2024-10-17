use crate::SkepMqttPlatform;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;
use serde_json::Value;
use skep_core::constants::EntityCategory;
use skep_sensor::SensorDeviceClass;

use crate::discovery::MQTTDiscoveryNew;
use bevy_ecs::{
    prelude::{Added, Commands},
    system::Query,
};
use lazy_static::lazy_static;
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
            .add_systems(Update, create_by_discovery_payload);
    }
}

const DOMAIN: &str = "sensor";

fn on_mqtt_platform_added(
    mut q_platform: Query<(Entity, &mut SkepMqttPlatform), (Added<SkepMqttPlatform>)>,
) {
    for (_entity, mut platform) in q_platform.iter_mut() {
        platform.platforms_loaded.insert(DOMAIN.to_string());
        // let id = commands
        //     .spawn((Name::new("sensor"), MQTTSupportComponent::Sensor))
        //
        //     .id();
        // commands.entity(entity).add_child(id);
    }
}

fn create_by_discovery_payload(mut create_ev: EventReader<MQTTDiscoveryNew>) {
    for ev in create_ev.read() {
        if ev.hash.component == DOMAIN {
            println!("mqtt sensor Create by discovery payload: {:#?}", ev);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MqttSensorConfiguration {
    pub availability_topic: Option<String>,
    // pub device: Option<DeviceInfo>,
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
