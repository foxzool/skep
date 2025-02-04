use crate::{
    binary_sensor::MqttBinarySensorPlugin,
    constants::DOMAIN,
    discovery::{
        on_mqtt_message_received, setup_new_entity_from_discovery, sub_default_topic,
        update_entity_from_discovery, MQTTDiscoveryHash, MQTTDiscoveryNew, MQTTDiscoveryPayload,
        MQTTDiscoveryUpdate, MQTTSupportComponent, ProcessDiscoveryPayload,
    },
    entity::{MQTTAvailability, MQTTAvailabilityConfiguration},
    sensor::MqttSensorPlugin,
    subscription::{add_state_subscription, update_available_subscription, MQTTStateSubscription},
};
use bevy_app::prelude::*;
use bevy_core::Name;
use bevy_ecs::prelude::*;
use bevy_mqtt::{rumqttc, MqttClientError, MqttConnectError, MqttPlugin, MqttSetting};
use bevy_reflect::Reflect;
use bevy_state::app::StatesPlugin;
use bevy_utils::{HashMap, HashSet};
use serde::Deserialize;
use serde_json::{Map, Value};
use skep_core::{
    integration::Integration, loader::LoadConfig, platform::Platform, typing::ConfigType,
    CallbackType,
};
use std::collections::VecDeque;

mod abbreviations;
mod binary_sensor;
mod constants;
mod discovery;
mod entity;
mod models;
mod sensor;
mod subscription;

type DiscoveryInfoType = Map<String, Value>;

pub struct SkepMqttPlugin;

impl Plugin for SkepMqttPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.add_plugins(MqttPlugin)
            .register_type::<SkepMqttPlatform>()
            .register_type::<MQTTDiscoveryHash>()
            .register_type::<MQTTSupportComponent>()
            .register_type::<MQTTAvailability>()
            .register_type::<MQTTStateSubscription>()
            .register_type::<HashSet<(String, String)>>()
            .add_event::<ProcessDiscoveryPayload>()
            .add_event::<MQTTDiscoveryNew>()
            .add_event::<MQTTDiscoveryUpdate>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    sub_default_topic,
                    on_mqtt_message_received,
                    handle_error,
                    add_state_subscription,
                    update_available_subscription,
                ),
            )
            .add_plugins((MqttSensorPlugin, MqttBinarySensorPlugin))
            .observe(reload_config);
    }
}

fn setup(mut _commands: Commands) {}

fn handle_error(
    mut connect_errors: EventReader<MqttConnectError>,
    mut client_errors: EventReader<MqttClientError>,
) {
    for error in connect_errors.read() {
        println!("connect Error: {:?}", error);
    }

    for error in client_errors.read() {
        println!("client Error: {:?}", error);
    }
}

#[derive(Debug, Component, Reflect)]
pub(crate) struct SkepMqttPlatform {
    #[reflect(ignore)]
    pub(crate) last_discovery: chrono::DateTime<chrono::Utc>,
    /// default discovery prefix topic: homeassistant
    pub discovery_prefix: String,
    pub discovered: HashMap<MQTTDiscoveryHash, Entity>,
    pub discovery_already_discovered: HashSet<MQTTDiscoveryHash>,
    #[reflect(ignore)]
    pub discovery_pending_discovered: HashMap<MQTTDiscoveryHash, PendingDiscovered>,
    #[reflect(ignore)]
    pub discovery_registry_hooks: HashMap<(String, String), CallbackType>,
    pub platforms_loaded: HashSet<String>,
    #[reflect(ignore)]
    pub config: Vec<ConfigType>,
}

impl Default for SkepMqttPlatform {
    fn default() -> Self {
        Self {
            last_discovery: Default::default(),
            discovery_prefix: "homeassistant".to_string(),
            discovered: Default::default(),
            discovery_already_discovered: Default::default(),
            discovery_pending_discovered: Default::default(),
            discovery_registry_hooks: Default::default(),
            platforms_loaded: Default::default(),
            config: vec![],
        }
    }
}

#[derive(Debug)]
pub struct PendingDiscovered {
    pub pending: VecDeque<MQTTDiscoveryPayload>,
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {
    /// mqtt broker host
    pub broker: String,
    pub client_key: Option<String>,
    pub client_cert: Option<String>,
    /// mqtt client transport
    pub transport: Option<String>,
    /// mqtt broker port
    pub port: u16,
    /// Request channel capacity
    pub capacity: Option<usize>,
    /// Whether to enable device auto discovery
    pub auto_discovery: Option<bool>,
    /// default discovery prefix topic: homeassistant
    pub discovery_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MqttLoader {
    mqtt_config_entry: Vec<MqttConfig>,
}

pub fn reload_config(trigger: Trigger<LoadConfig>, mut commands: Commands) {
    let binding = trigger.event().config.clone();
    let config_value = binding.as_object().unwrap();

    if let Some(mqtt_config) = config_value.get(DOMAIN) {
        if let Some(mqtt_config_list) = mqtt_config.as_object() {
            let config = serde_json::from_value::<ConfigType>(mqtt_config.clone()).ok();

            println!("config: {:?}", config);
        }
        if let Ok(config) = serde_json::from_value::<MqttLoader>(mqtt_config.clone()) {
            for config_entry in config.mqtt_config_entry {
                let mut mqtt_options = rumqttc::MqttOptions::new(
                    "skep-client",
                    &config_entry.broker,
                    config_entry.port,
                );
                if let (Some(username), Some(password)) =
                    (config_entry.client_key, config_entry.client_cert)
                {
                    mqtt_options.set_credentials(username, password);
                }
                let transport = match config_entry.transport {
                    None => rumqttc::Transport::Tcp,
                    Some(s) => match s.as_str() {
                        "tcp" => rumqttc::Transport::Tcp,
                        "ws" | "websocket" => rumqttc::Transport::Ws,
                        _ => rumqttc::Transport::Tcp,
                    },
                };

                mqtt_options.set_transport(transport);

                commands
                    .spawn((
                        Name::new("MQTT".to_string()),
                        Integration {
                            name: "MQTT".to_string(),
                            domain: "mqtt".to_string(),
                        },
                        Platform::new(format!("{}:{}", config_entry.broker, config_entry.port)),
                        MqttSetting {
                            mqtt_options,
                            cap: 20,
                        },
                        SkepMqttPlatform::default(),
                    ))
                    .observe(setup_new_entity_from_discovery)
                    .observe(update_entity_from_discovery);
            }
        }
    }
}

pub fn on_setup_config_entry() {}
