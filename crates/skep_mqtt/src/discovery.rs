use crate::{
    abbreviations::{ABBREVIATIONS, DEVICE_ABBREVIATIONS, ORIGIN_ABBREVIATIONS},
    PendingDiscovered, SkepMqttPlatform,
};
use anyhow::Context;
use bevy_ecs::prelude::*;
use bevy_log::{debug, trace, warn};
use bevy_mqtt::{
    rumqttc::{QoS, SubscribeFilter},
    MqttClient, MqttClientConnected, MqttPublishPacket,
};

use crate::{
    entity::{AvailabilityConfig, MQTTAvailability, MQTTAvailabilityConfiguration},
    subscription::MQTTStateSubscription,
};
use bevy_core::Name;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::{ComponentHooks, StorageType},
    system::EntityCommands,
    world::DeferredWorld,
};
use bevy_hierarchy::{BuildChildren, Parent};
use bevy_reflect::Reflect;
use bevy_utils::HashMap;
use regex::{Error, Regex};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use skep_core::{
    constants::EntityCategory,
    device::DeviceEntry,
    helper::{device_registry::DeviceInfo, entity::SkepEntityComponent},
    typing::SetupConfigEntry,
};
use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
    str::FromStr,
};
use strum_macros::{Display, EnumString};

#[derive(Debug, Component, EnumString, Display, Clone, PartialEq, Eq, Hash, Reflect)]
#[strum(serialize_all = "snake_case")]
#[strum(ascii_case_insensitive)]
pub enum MQTTSupportComponent {
    Sensor,
}

#[derive(Debug, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct MQTTDiscoveryHash {
    pub component: String,
    pub discovery_id: String,
}

impl Display for MQTTDiscoveryHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.component, self.discovery_id)
    }
}

impl Component for MQTTDiscoveryHash {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world: DeferredWorld, entity, _component_id| {
            let discovery_hash = world.get::<MQTTDiscoveryHash>(entity).unwrap();
            let component_name = discovery_hash.component.clone();
            let name = Name::new(format!(
                "{} {}",
                discovery_hash.component, discovery_hash.discovery_id
            ));
            let mut commands = world.commands();
            let mut binding = commands.entity(entity);
            let mut e = binding.insert(name);
            if let Ok(mqtt_component) = MQTTSupportComponent::from_str(&component_name) {
                e.insert(mqtt_component);
            } else {
                warn!("Component not supported: {}", component_name);
            }
        });
    }
}

#[derive(Debug, Component, Clone)]
pub struct MQTTDiscoveryPayload {
    pub topic: String,
    pub payload: Map<String, Value>,
}

/// Subscribe to default topic
pub fn sub_default_topic(
    mqtt_clients: Query<(&SkepMqttPlatform, &MqttClient), Added<MqttClientConnected>>,
) {
    for (mqtt_platform, mqtt_client) in mqtt_clients.iter() {
        let mut subs = vec![];
        for component in SUPPORTED_COMPONENTS {
            subs.push(SubscribeFilter::new(
                format!("{}/{}/+/config", mqtt_platform.discovery_prefix, component),
                QoS::AtMostOnce,
            ));

            subs.push(SubscribeFilter::new(
                format!(
                    "{}/{}/+/+/config",
                    mqtt_platform.discovery_prefix, component
                ),
                QoS::AtMostOnce,
            ));
        }
        mqtt_client.subscribe_many(subs).unwrap();
    }
}

pub(crate) const SUPPORTED_COMPONENTS: &[&str] = &[
    "alarm_control_panel",
    "binary_sensor",
    "button",
    "camera",
    "climate",
    "cover",
    "device_automation",
    "device_tracker",
    "event",
    "fan",
    "humidifier",
    "image",
    "lawn_mower",
    "light",
    "lock",
    "notify",
    "number",
    "scene",
    "siren",
    "select",
    "sensor",
    "switch",
    "tag",
    "text",
    "update",
    "vacuum",
    "valve",
    "water_heater",
];

#[derive(Event, Clone)]
pub struct ProcessDiscoveryPayload {
    pub component: String,
    pub object_id: String,
    pub payload: HashMap<String, Value>,
}

pub(crate) fn on_mqtt_message_received(
    mut publish_ev: EventReader<MqttPublishPacket>,
    mut query: Query<&mut SkepMqttPlatform>,
    mut commands: Commands,
) {
    for packet in publish_ev.read() {
        if let Ok(mut mqtt_platform) = query.get_mut(packet.entity) {
            mqtt_platform.last_discovery = chrono::Utc::now();

            let payload = packet.payload.clone();
            let topic = packet.topic.clone();
            debug!("topic: {} received : {:?}", topic, payload.len());
            let topic_trimmed = topic.replacen(
                format!("{}/", mqtt_platform.discovery_prefix).as_str(),
                "",
                1,
            );

            if let Ok((component, node_id, object_id)) = parse_topic_config(&topic_trimmed) {
                if let Ok(discovery_payload) = handle_discovery_message(&payload) {
                    if let Ok(components) = serde_json::from_value::<MQTTDiscoveryComponents>(
                        Value::from(discovery_payload.clone()),
                    ) {
                        let discovery_id = if let Some(node_id) = node_id {
                            format!("{} {}", node_id, object_id)
                        } else {
                            object_id.clone()
                        };
                        let discovery_hash = MQTTDiscoveryHash {
                            component: component.clone(),
                            discovery_id: discovery_id.clone(),
                        };

                        let discovery_payload = MQTTDiscoveryPayload {
                            topic: topic_trimmed.to_string(),
                            payload: discovery_payload,
                        };

                        let mut is_new = false;
                        let entity =
                            if let Some(entity) = mqtt_platform.discovered.get(&discovery_hash) {
                                debug!(
                                    "Component has already been discovered: {}, queuing update",
                                    discovery_hash
                                );
                                *entity
                            } else {
                                is_new = true;
                                let id = commands.spawn_empty().id();
                                commands.entity(packet.entity).add_child(id);
                                mqtt_platform.discovered.insert(discovery_hash.clone(), id);
                                id
                            };

                        let mut cmds = commands.entity(entity);
                        if is_new {
                            cmds.insert(discovery_hash).insert(discovery_payload);
                        } else {
                            cmds.insert(discovery_payload);
                        }

                        spawn_or_update_components(&mut cmds, components);
                    }
                }
            }
        } else {
            warn!("MqttPlatform not found");
        }
    }
}

#[derive(Bundle)]
struct DiscoveryDefaultBundle {
    discovery_hash: MQTTDiscoveryHash,
    discovery_payload: MQTTDiscoveryPayload,
    state_subscription: MQTTStateSubscription,
}

fn handle_discovery_message(payload: &[u8]) -> anyhow::Result<Map<String, Value>> {
    let discovery_payload = match serde_json::from_slice::<Value>(payload) {
        Err(_) => serde_json::Map::new(),
        Ok(mut json_data) => {
            let mut discovery_payload = json_data
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("Expected a JSON object"))?;
            replace_all_abbreviations(&mut discovery_payload)?;
            if !valid_origin_info(&discovery_payload) {
                return Err(anyhow::anyhow!("Invalid origin info"));
            }

            if discovery_payload.contains_key(TOPIC_BASE) {
                replace_topic_base(&mut discovery_payload);
            }

            discovery_payload
        }
        .to_owned(),
    };

    Ok(discovery_payload)
}

/// Spawn or Update MQTT components from discovery payload
fn spawn_or_update_components(cmds: &mut EntityCommands, components: MQTTDiscoveryComponents) {
    if let Some(availability_config) = components.availability_config {
        let availability = MQTTAvailability::from_config(availability_config);

        if !availability.topic.is_empty() {
            cmds.insert(availability);
        }
    }

    if let Some(entity_category) = components.entity_category {
        cmds.insert(entity_category);
    }

    if let Some(device_info) = components.device {
        cmds.insert(device_info);
    }
}

#[derive(Deserialize)]
struct MQTTDiscoveryComponents {
    #[serde(flatten)]
    state_subscription: MQTTStateSubscription,
    #[serde(flatten)]
    availability_config: Option<MQTTAvailabilityConfiguration>,
    #[serde(flatten)]
    entity_category: Option<EntityCategory>,
    device: Option<DeviceInfo>,
}

// Replace all abbreviations in the payload
fn replace_all_abbreviations(discovery_payload: &mut Map<String, Value>) -> anyhow::Result<()> {
    replace_abbreviations(discovery_payload, &ABBREVIATIONS);

    if let Some(origin_json) = discovery_payload.get_mut("origin") {
        if let Some(origin) = origin_json.as_object_mut() {
            replace_abbreviations(origin, &ORIGIN_ABBREVIATIONS);
        }
    }

    if let Some(device) = discovery_payload.get_mut("device") {
        if let Some(device) = device.as_object_mut() {
            replace_abbreviations(device, &DEVICE_ABBREVIATIONS);
        }
    }

    if let Some(availability) = discovery_payload.get_mut(CONF_AVAILABILITY) {
        if let Some(list) = availability.as_array_mut() {
            for item in list {
                if let Some(item) = item.as_object_mut() {
                    replace_abbreviations(item, &ABBREVIATIONS);
                }
            }
        }
    }

    Ok(())
}
fn replace_abbreviations(
    json: &mut Map<String, Value>,
    abbreviations: &HashMap<&'static str, &'static str>,
) {
    // Collect keys beforehand to avoid invalidating iterators during modification.
    let keys: Vec<_> = json.keys().cloned().collect();

    for key in keys {
        if let Some(full_name) = abbreviations.get(&key.as_str()) {
            if let Some(value) = json.remove(&key) {
                json.insert(full_name.to_string(), value);
            }
        }
    }
}
/// Parse topic to get component, node_id and object_id
fn parse_topic_config(topic: &str) -> Result<(String, Option<String>, String), Error> {
    let regex = Regex::new(
        r"(?P<component>\w+)/(?:(?P<node_id>[a-zA-Z0-9_-]+)/)?(?P<object_id>[a-zA-Z0-9_-]+)/config",
    )?;

    if let Some(captures) = regex.captures(topic) {
        let component = captures["component"].to_string();
        let node_id = captures.name("node_id").map(|m| m.as_str().to_string());
        let object_id = captures["object_id"].to_string();
        Ok((component, node_id, object_id))
    } else {
        Err(Error::Syntax("Invalid topic".to_string()))
    }
}

fn valid_origin_info(payload: &Map<String, Value>) -> bool {
    if let Some(origin) = payload.get("origin") {
        if let Err(e) = serde_json::from_value::<MqttOriginInfo>(origin.clone()) {
            warn!("origin info error: {}, {}", origin.to_string(), e)
        }
    }
    true
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MqttOriginInfo {
    name: String,
    sw_version: Option<String>,
    support_url: Option<String>,
}

const TOPIC_BASE: &str = "~";
const CONF_AVAILABILITY: &str = "availability";
const CONF_TOPIC: &str = "topic";

/// Replace topic base in MQTT discovery data.
fn replace_topic_base(discovery_payload: &mut Map<String, Value>) {
    if let Some(base) = discovery_payload.remove(TOPIC_BASE) {
        if let Value::String(base_str) = base {
            for (key, value) in discovery_payload.iter_mut() {
                match value {
                    Value::String(ref mut val_str) if !val_str.is_empty() => {
                        if key.ends_with("topic") && val_str.starts_with(TOPIC_BASE) {
                            *val_str = format!("{}{}", base_str, &val_str[TOPIC_BASE.len()..]);
                        } else if key.ends_with("topic") && val_str.ends_with(TOPIC_BASE) {
                            *val_str = format!(
                                "{}{}",
                                &val_str[..val_str.len() - TOPIC_BASE.len()],
                                base_str
                            );
                        }
                    }
                    _ => {}
                }
            }

            if let Some(availability) = discovery_payload.get_mut(CONF_AVAILABILITY) {
                if let Value::Array(ref mut availability_list) = availability {
                    for availability_conf in availability_list.iter_mut() {
                        if let Value::Object(ref mut conf_map) = availability_conf {
                            if let Some(topic) = conf_map.get_mut(CONF_TOPIC) {
                                if let Value::String(ref mut topic_str) = topic {
                                    if topic_str.starts_with(TOPIC_BASE) {
                                        *topic_str = format!(
                                            "{}{}",
                                            base_str,
                                            &topic_str[TOPIC_BASE.len()..]
                                        );
                                    } else if topic_str.ends_with(TOPIC_BASE) {
                                        *topic_str = format!(
                                            "{}{}",
                                            &topic_str[..topic_str.len() - TOPIC_BASE.len()],
                                            base_str
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn device_info_from_payload(payload: Map<String, Value>) -> anyhow::Result<Option<DeviceInfo>> {
    match payload.get("device").cloned() {
        None => Ok(None),
        Some(device_value) => serde_json::from_value(device_value)
            .with_context(|| "Failed to deserialize device info"),
    }
}

#[test]
fn test_replace_base_topic() {
    let mut json = json!({
       "~":"homeassistant/switch/irrigation",
       "name":"garden",
       "cmd_t":"~/set",
       "stat_t":"~/state"
    });

    let mut json_data = json.as_object_mut().unwrap();
    replace_all_abbreviations(&mut json_data).unwrap();
    replace_topic_base(&mut json_data);
    assert_eq!(
        json!(json_data),
        json!({
           "name":"garden",
           "command_topic":"homeassistant/switch/irrigation/set",
           "state_topic":"homeassistant/switch/irrigation/state"
        })
    );
}

#[test]
fn test_availability() {
    let mut json = json!({"availability_topic":"watermeter/connection","device":{"configuration_url":"http://192.168.1.51","identifiers":["watermeter"],"manufacturer":"AI on the Edge Device","model":"Meter Digitizer","name":"watermeter","sw_version":"v15.7.0"},"device_class":"problem","icon":"mdi:alert-outline","name":"Problem","object_id":"watermeter_problem","payload_available":"connected","payload_not_available":"connection lost","state_topic":"watermeter/main/error","unique_id":"watermeter-problem","value_template":"{{ 'OFF' if 'no error' in value else 'ON'}}"});

    let availability = serde_json::from_value::<MQTTAvailability>(json.clone()).unwrap();
}
