use crate::discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload};
use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    observer::Trigger,
    prelude::{Added, Query},
};
use bevy_log::debug;
use serde::{Deserialize, Serialize};
use skep_core::typing::SetupConfigEntry;

pub struct MqttBinarySensorPlugin;

impl Plugin for MqttBinarySensorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_or_update_discovery_payload)
            .observe(on_setup_entry);
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
