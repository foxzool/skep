use crate::{
    binary_sensor::MqttBinarySensorPlugin,
    constants::DOMAIN,
    discovery::{
        on_discovery_message_received, process_discovery_payload, sub_default_topic,
        ProcessDiscoveryPayload,
    },
    sensor::MqttSensorPlugin,
};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_mqtt::{rumqttc, MqttClientError, MqttConnectError, MqttPlugin, MqttSetting};
use bevy_reflect::Reflect;
use bevy_state::app::StatesPlugin;
use bevy_utils::HashSet;
use serde::Deserialize;
use serde_json::{Map, Value};
use skep_core::{
    integration::Integration, loader::LoadConfig, platform::Platform, typing::ConfigType,
};
use std::collections::{HashMap, VecDeque};

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
            .register_type::<HashSet<(String, String)>>()
            .add_event::<ProcessDiscoveryPayload>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    sub_default_topic,
                    on_discovery_message_received,
                    handle_error,
                ),
            )
            .add_plugins((MqttSensorPlugin, MqttBinarySensorPlugin))
            .observe(reload_config)
            .observe(process_discovery_payload);
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
    pub discovery_already_discovered: HashSet<(String, String)>,
    #[reflect(ignore)]
    pub discovery_pending_discovered: HashMap<(String, String), PendingDiscovered>,
    pub platforms_loaded: HashSet<String>,
    #[reflect(ignore)]
    pub config: Vec<ConfigType>,
}

impl Default for SkepMqttPlatform {
    fn default() -> Self {
        Self {
            last_discovery: Default::default(),
            discovery_prefix: "homeassistant".to_string(),
            discovery_already_discovered: Default::default(),
            discovery_pending_discovered: Default::default(),
            platforms_loaded: Default::default(),
            config: vec![],
        }
    }
}

#[derive(Debug)]
pub struct PendingDiscovered {
    pub pending: VecDeque<Map<String, Value>>,
}

impl PendingDiscovered {
    pub fn new(pending: VecDeque<Map<String, Value>>) -> Self {
        Self { pending }
    }
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

                commands.spawn((
                    Integration::new("MQTT"),
                    Platform::new(format!("{}:{}", config_entry.broker, config_entry.port)),
                    MqttSetting {
                        mqtt_options,
                        cap: 20,
                    },
                    SkepMqttPlatform::default(),
                ));
            }
        }
    }
}

pub fn on_setup_config_entry() {}
