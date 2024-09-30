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

use regex::{Error, Regex};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use skep_core::{
    helper::{device_registry::DeviceInfo, entity::SkepEntityComponent},
    typing::SetupConfigEntry,
};
use std::collections::{HashMap, VecDeque};

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
    pub payload: Map<String, Value>,
}

pub(crate) fn on_discovery_message_received(
    mut publish_ev: EventReader<MqttPublishPacket>,
    mut query: Query<&mut SkepMqttPlatform>,
    mut commands: Commands,
) {
    for packet in publish_ev.read() {
        if let Ok(mut mqtt_platform) = query.get_mut(packet.entity) {
            trace!("topic: {} received : {:?}", packet.topic, packet.payload);
            mqtt_platform.last_discovery = chrono::Utc::now();

            let payload = packet.payload.clone();
            let topic = packet.topic.clone();
            let topic_trimmed = topic.replacen(
                format!("{}/", mqtt_platform.discovery_prefix).as_str(),
                "",
                1,
            );
            match handle_discovery_message(&topic_trimmed, &payload) {
                Ok(Some(event)) => {
                    commands.trigger_targets(event, vec![packet.entity]);
                }
                Ok(None) => {}
                Err(e) => {
                    if topic_trimmed.ends_with("config") {
                        warn!("handle discovery message error: {:?}", e);
                    }
                }
            }
        } else {
            warn!("MqttPlatform not found");
        }
    }
}

fn handle_discovery_message(
    // mut mqtt_res: ResMut<MqttResource>,
    topic: &str,
    payload: &[u8],
) -> anyhow::Result<Option<ProcessDiscoveryPayload>> {
    let (component, node_id, object_id) = parse_topic_config(topic)?;

    let mut discovery_payload = match serde_json::from_slice::<Value>(payload) {
        Err(_) => serde_json::Map::new(),
        Ok(mut json_data) => {
            let mut discovery_payload = json_data
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("Expected a JSON object"))?;
            replace_all_abbreviations(&mut discovery_payload)?;
            if !valid_origin_info(&discovery_payload) {
                return Ok(None);
            }

            if discovery_payload.contains_key(TOPIC_BASE) {
                replace_topic_base(&mut discovery_payload);
            }

            discovery_payload
        }
        .to_owned(),
    };

    let discovery_id = if let Some(node_id) = node_id {
        format!("{} {}", node_id, object_id)
    } else {
        object_id.clone()
    };
    let discovery_hash = (component.to_string(), discovery_id.clone());

    if !discovery_payload.is_empty() {
        let discovery_data = json!({
            "discovery_hash": discovery_hash,
            "discovery_topic": topic,
            "discovery_payload": serde_json::from_slice::<Value>(payload)?,
        });
        discovery_payload.insert("discovery_data".to_string(), discovery_data);
        discovery_payload.insert("platform".to_string(), Value::String("mqtt".to_string()));
    }

    let trigger = ProcessDiscoveryPayload {
        component,
        object_id: discovery_id.clone(),
        payload: discovery_payload,
    };

    Ok(Some(trigger))
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

fn device_info_from_payload(payload: Map<String, Value>) -> anyhow::Result<Option<DeviceInfo>> {
    match payload.get("device").cloned() {
        None => Ok(None),
        Some(device_value) => serde_json::from_value(device_value)
            .with_context(|| "Failed to deserialize device info"),
    }
}

pub(crate) fn process_discovery_payload(
    mut trigger: Trigger<ProcessDiscoveryPayload>,
    mut query: Query<&mut SkepMqttPlatform>,
    mut commands: Commands,
) {
    let event = trigger.event().clone();
    let mut mqtt_platform = query.get_mut(trigger.entity()).unwrap();

    let ProcessDiscoveryPayload {
        component,
        object_id: discovery_id,
        payload,
    } = &trigger.event();
    let discovery_hash = (component.to_string(), discovery_id.to_string());

    if let Some(pending) = mqtt_platform
        .discovery_pending_discovered
        .get_mut(&discovery_hash)
    {
        pending.pending.push_front(payload.clone());
        debug!(
            "Component has already been discovered: {} {}, queuing update",
            component, discovery_id,
        );
        return;
    }

    trace!("Process discovery payload {:?}", payload);

    let already_discovered = mqtt_platform
        .discovery_already_discovered
        .contains(&discovery_hash);
    if (already_discovered || !payload.is_empty())
        && !mqtt_platform
            .discovery_pending_discovered
            .contains_key(&discovery_hash)
    {
        mqtt_platform.discovery_pending_discovered.insert(
            discovery_hash.clone(),
            PendingDiscovered::new(VecDeque::new()),
        );
    }

    if !mqtt_platform.platforms_loaded.contains(component) {
        debug!("{:?} waiting setup ", discovery_hash);
        // if let Ok(Some(device_info)) = device_info_from_payload(payload.clone()) {}
        commands.trigger_targets(
            SetupConfigEntry {
                component: event.component,
                object_id: event.object_id,
                payload: event.payload.into(),
            },
            vec![trigger.entity()],
        );
    }
}
