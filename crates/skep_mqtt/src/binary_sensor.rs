use crate::{
    discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload},
    SkepMqttPlatform,
};
use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    observer::Trigger,
    prelude::{Added, Commands, Query},
};
use bevy_log::debug;
use serde::{Deserialize, Serialize};
use skep_core::typing::SetupConfigEntry;

pub struct MqttBinarySensorPlugin;

impl Plugin for MqttBinarySensorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_or_update_discovery_payload, on_mqtt_platform_added),
        )
        .observe(on_setup_entry);
    }
}

fn on_mqtt_platform_added(mut q_platform: Query<&mut SkepMqttPlatform, (Added<SkepMqttPlatform>)>) {
    for mut platform in q_platform.iter_mut() {
        platform.platforms_loaded.insert(DOMAIN.to_string());
    }
}

const DOMAIN: &str = "binary_sensor";

fn on_setup_entry(trigger: Trigger<SetupConfigEntry>) {
    if trigger.event().component == DOMAIN {
        println!("Setup binary sensor");
    }
}

fn create_or_update_discovery_payload(
    q_discovery: Query<(&MQTTDiscoveryHash, &MQTTDiscoveryPayload), Added<MQTTDiscoveryHash>>,
) {
    for (hash, payload) in q_discovery.iter() {
        if hash.component == DOMAIN {
            debug!("create_or_update_discovery_payload {:#?}", payload);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MqttBinarySensorConfiguration {}
