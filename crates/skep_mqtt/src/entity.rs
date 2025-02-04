use crate::{
    constants::{
        CONF_AVAILABILITY, CONF_AVAILABILITY_TEMPLATE, CONF_AVAILABILITY_TOPIC,
        CONF_ENABLED_BY_DEFAULT, CONF_OBJECT_ID, CONF_PAYLOAD_AVAILABLE,
        CONF_PAYLOAD_NOT_AVAILABLE, CONF_TOPIC,
    },
    discovery::MQTTDiscoveryPayload,
    subscription::MQTTStateSubscription,
    DiscoveryInfoType,
};
use bevy_core::Name;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::Trigger,
    prelude::{Commands, In, Query, ResMut, System},
};
use bevy_log::debug;
use bevy_mqtt::TopicMessage;
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize};
use bevy_utils::HashMap;
use minijinja::{context, Environment, Template};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{
        EntityCategory, CONF_DEVICE, CONF_ENTITY_CATEGORY, CONF_ICON, CONF_NAME, CONF_UNIQUE_ID,
        CONF_VALUE_TEMPLATE,
    },
    helper::entity::{SkepEntity, SkepEntityComponent},
    states::State,
    typing::ConfigType,
    CallbackType, SkepResource,
};
use std::{
    cmp::PartialEq,
    str::FromStr,
    sync::{Arc, RwLock},
};

#[derive(Debug, Default)]
pub struct MqttEntityComponent {
    device_specifications: Option<HashMap<String, Value>>,
    config: ConfigType,
    // sub_state: HashMap<String, EntitySubscription>,
    discovery: bool,
    subscriptions: HashMap<String, HashMap<String, Value>>,
    default_name: Option<String>,
    entity_id_format: String,
}

impl MqttEntityComponent {
    pub fn new(
        skep_res: &ResMut<SkepResource>,
        config: ConfigType,
        config_entry: ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) -> anyhow::Result<(SkepEntityComponent, MqttEntityComponent)> {
        let mut skep_entity = SkepEntityComponent::default();
        let mut mqtt_entity = MqttEntityComponent::default();
        mqtt_entity.config = config.clone();
        skep_entity.unique_id = config
            .get(CONF_UNIQUE_ID)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        mqtt_entity.discovery = discovery_data.is_some();

        mqtt_entity.setup_common_attributes_from_config(&mut skep_entity, config);
        mqtt_entity.init_entity_id();

        Ok((skep_entity, mqtt_entity))
    }

    fn setup_common_attributes_from_config(
        &mut self,
        skep_entity: &mut SkepEntityComponent,
        config: ConfigType,
    ) {
        skep_entity.entity_category = config.get(CONF_ENTITY_CATEGORY).and_then(|v| {
            v.as_str().and_then(|s| {
                EntityCategory::from_str(s)
                    .map_err(|_| anyhow::anyhow!("Invalid entity category: {}", s))
                    .ok()
            })
        });
        skep_entity.entity_registry_enabled_default = config
            .get(CONF_ENABLED_BY_DEFAULT)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        skep_entity.icon = config
            .get(CONF_ICON)
            .and_then(|v| v.as_str().map(|s| s.to_string()));
    }

    fn set_entity_name(&mut self, skep_entity: &mut SkepEntityComponent, config: ConfigType) {
        match config.get(CONF_NAME) {
            Some(entity_name) => {
                skep_entity.name = entity_name.as_str().map(|s| s.to_string());
            }
            None => {
                if !self.default_to_device_class_name() {
                    skep_entity.name = self.default_name.clone();
                } else {
                    skep_entity.name = None;
                }
            }
        }

        if let Some(device) = config.get(CONF_DEVICE).and_then(|v| v.as_object()) {
            if !device.contains_key(CONF_NAME) {
                log::info!(
                    "MQTT device information always needs to include a name, got {:?}, \
if device information is shared between multiple entities, the device \
name must be included in each entity's device configuration",
                    config
                );
            }
        }
    }

    fn default_to_device_class_name(&self) -> bool {
        false
    }

    fn init_entity_id(&mut self) {}
}

pub trait MqttDiscoveryUpdateMixin: SkepEntity {
    fn init_discovery_update(
        &mut self,
        discovery_data: Option<DiscoveryInfoType>,
        discovery_update: Option<Box<dyn System<In = In<MQTTDiscoveryPayload>, Out = ()>>>,
    ) {
        self.set_discovery_data(discovery_data.clone());
        self.set_discovery_update(discovery_update);
        self.set_remove_discovery_update(None);
        if discovery_data.is_none() {
            return;
        }
    }

    fn set_discovery_data(&mut self, discovery_data: Option<DiscoveryInfoType>);

    fn set_discovery_update(
        &mut self,
        discovery_update: Option<Box<dyn System<In = In<MQTTDiscoveryPayload>, Out = ()>>>,
    );

    fn set_remove_discovery_update(&mut self, callback_type: Option<CallbackType>);

    fn get_device_specifications(&self) -> Option<&HashMap<String, Value>>;

    fn get_config_entry(&self) -> &ConfigEntry;
}

pub trait MqttEntityDeviceInfo: SkepEntity {
    fn init(specifications: Option<HashMap<String, Value>>, config_entry: ConfigEntry) -> Self;
}

pub trait MqttEntity: MqttDiscoveryUpdateMixin + MqttEntityDeviceInfo {
    fn default_name(&self) -> Option<String>;

    fn get_attr_force_update(&self) -> bool {
        false
    }

    fn entity_id_format(&self) -> &str;

    fn get_attr_has_entity_name(&self) -> bool {
        true
    }

    fn should_poll(&self) -> bool {
        false
    }

    fn setup_common_attributes_from_config(&mut self, config: &ConfigType) {
        let entity_category = config.get(CONF_ENTITY_CATEGORY).and_then(|v| {
            v.as_str().and_then(|s| {
                EntityCategory::from_str(s)
                    .map_err(|_| anyhow::anyhow!("Invalid entity category: {}", s))
                    .ok()
            })
        });
        self.set_entity_category(entity_category);

        let entity_registry_enabled_default = config
            .get(CONF_ENABLED_BY_DEFAULT)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.set_entity_registry_enabled_default(entity_registry_enabled_default);

        let icon = config
            .get(CONF_ICON)
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        self.set_icon(icon);
        self.set_entity_name(config);
    }

    fn default_to_device_class_name(&self) -> bool {
        false
    }

    fn set_entity_name(&mut self, config: &ConfigType) {
        let name = match config.get(CONF_NAME) {
            Some(entity_name) => entity_name.as_str().map(|s| s.to_string()),
            None => {
                if !self.default_to_device_class_name() {
                    self.default_name()
                } else {
                    None
                }
            }
        };

        if let Some(device) = config.get(CONF_DEVICE).and_then(|v| v.as_object()) {
            if !device.contains_key(CONF_NAME) {
                log::info!(
                    "MQTT device information always needs to include a name, got {:?}, \
if device information is shared between multiple entities, the device \
name must be included in each entity's device configuration",
                    config
                );
            }
        }
    }

    fn config(&self) -> &ConfigType;

    fn set_config(&mut self, config: ConfigType);
    fn set_discovery(&mut self, discovery_data: Option<DiscoveryInfoType>);

    fn init_mqtt_entity(
        &mut self,
        mut skep_res: ResMut<SkepResource>,
        config: &ConfigType,
        config_entry: &ConfigEntry,
        discovery_data: Option<DiscoveryInfoType>,
    ) {
        self.set_config(config.clone());
        let unique_id = config
            .get(CONF_UNIQUE_ID)
            .map(|v| v.as_str().unwrap().to_string());
        self.set_unique_id(unique_id);
        self.set_discovery(discovery_data);

        self.setup_common_attributes_from_config(&config);
        self.init_entity_id(&mut skep_res);
    }

    fn init_entity_id(&mut self, skep_res: &mut ResMut<SkepResource>) {
        println!("config {:#?}", self.config());
        self.init_entity_id_from_config(skep_res).unwrap();
    }

    fn init_entity_id_from_config(
        &mut self,
        skep_res: &mut ResMut<SkepResource>,
    ) -> anyhow::Result<()> {
        if let Some(object_id) = self.config().get(CONF_OBJECT_ID) {
            let current_ids = skep_res.entity_ids.clone().into_iter().collect();
            let entity_id = skep_res.generate_entity_id(
                self.entity_id_format(),
                object_id.as_str(),
                Some(current_ids),
            )?;
            self.set_entity_id(Some(entity_id));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MQTTAvailabilityConfiguration {
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
    pub availability: Option<Vec<AvailabilityConfig>>,
    pub availability_topic: Option<String>,
    pub availability_template: Option<String>,
    pub availability_mode: Option<MQTTAvailabilityMode>,
}

#[derive(Debug, Serialize, Deserialize, Reflect, Clone)]
pub struct AvailabilityConfig {
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
    pub topic: String,
    pub value_template: Option<String>,
}

impl AvailabilityConfig {
    pub fn payload_available(&self) -> &str {
        self.payload_available.as_deref().unwrap_or("online")
    }

    pub fn payload_not_available(&self) -> &str {
        self.payload_not_available.as_deref().unwrap_or("offline")
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Reflect, Clone)]
#[reflect(PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MQTTAvailabilityMode {
    All,
    Any,
    #[default]
    Latest,
}

#[derive(Debug, Serialize, Deserialize, Component, Default, Reflect)]
pub struct MQTTAvailability {
    pub topic: HashMap<String, AvailabilityConfig>,
    pub avail_topics: HashMap<String, bool>,
    pub available_latest: bool,
    pub availability_model: MQTTAvailabilityMode,
}

impl MQTTAvailability {
    pub fn available(&self) -> bool {
        if self.avail_topics.is_empty() {
            true
        } else if self.availability_model == MQTTAvailabilityMode::All {
            self.avail_topics.values().all(|v| *v)
        } else if self.availability_model == MQTTAvailabilityMode::Any {
            self.avail_topics.values().any(|v| *v)
        } else {
            self.available_latest
        }
    }

    pub fn from_config(config: MQTTAvailabilityConfiguration) -> Self {
        let mut topic = HashMap::new();
        if let Some(availability_topic) = config.availability_topic {
            let avail_config = AvailabilityConfig {
                payload_available: config.payload_available,
                payload_not_available: config.payload_not_available,
                topic: availability_topic.clone(),
                value_template: config.availability_template,
            };
            topic.insert(availability_topic, avail_config);
        }

        if let Some(availability) = config.availability {
            for avail in availability {
                let avail_config = AvailabilityConfig {
                    payload_available: avail.payload_available,
                    payload_not_available: avail.payload_not_available,
                    topic: avail.topic.clone(),
                    value_template: avail.value_template,
                };
                topic.insert(avail.topic, avail_config);
            }
        }

        Self {
            topic,
            avail_topics: Default::default(),
            available_latest: false,
            availability_model: config.availability_mode.unwrap_or_default(),
        }
    }

    pub fn update_from_config(&mut self, update_config: MQTTAvailabilityConfiguration) {
        if let Some(mode) = update_config.availability_mode {
            self.availability_model = mode;
        }

        self.topic.clear();
        if let Some(availability_topic) = update_config.availability_topic {
            let avail_config = AvailabilityConfig {
                payload_available: update_config.payload_available,
                payload_not_available: update_config.payload_not_available,
                topic: availability_topic.clone(),
                value_template: update_config.availability_template,
            };
            self.topic.insert(availability_topic, avail_config);
        }

        if let Some(availability) = update_config.availability {
            for avail in availability {
                let avail_config = AvailabilityConfig {
                    payload_available: avail.payload_available,
                    payload_not_available: avail.payload_not_available,
                    topic: avail.topic.clone(),
                    value_template: avail.value_template,
                };
                self.topic.insert(avail.topic, avail_config);
            }
        }
    }
}

pub(crate) fn handle_state_value(
    topic_message: Trigger<TopicMessage>,
    mut commands: Commands,
    mut q_state_sub: Query<(Entity, &MQTTStateSubscription, &Name, Option<&mut State>)>,
) {
    if let Ok((entity, state_sub, name, mut opt_state)) =
        q_state_sub.get_mut(topic_message.entity())
    {
        if topic_message.event().topic == state_sub.state_topic {
            let update_state =
                try_render_template(&state_sub.value_template, &topic_message.event().payload)
                    .unwrap_or_default();
            if let Some(mut state) = opt_state {
                state.update(&update_state);
            } else {
                let state = State::new(update_state.clone());
                commands.entity(entity).insert(state);
            }

            if !update_state.is_empty() {
                debug!("{}: {}", name, update_state);
            }
        }
    }
}

pub(crate) fn try_render_template(
    value_template: &Option<String>,
    value_bytes: &[u8],
) -> anyhow::Result<String> {
    if let Some(value_template) = value_template {
        let mut env = Environment::new();
        env.add_template("value", &value_template)?;
        let template = env.get_template("value")?;

        let template_value = if let Ok(state_json) = serde_json::from_slice::<Value>(value_bytes) {
            template.render(context! { value_json => state_json })?
        } else {
            template.render(context! { value => value_bytes })?
        };

        Ok(template_value)
    } else {
        Ok(std::str::from_utf8(value_bytes)?.to_string())
    }
}

pub(crate) fn handle_available_value(
    topic_message: Trigger<TopicMessage>,
    mut q_avail: Query<(&mut MQTTAvailability, &Name)>,
) {
    if let Ok((mut available, name)) = q_avail.get_mut(topic_message.entity()) {
        if available.topic.get(&topic_message.event().topic).is_none() {
            return;
        }
        let update_status = try_render_available(&available, &topic_message.event()).ok();
        if let Some(update_status) = update_status {
            available
                .avail_topics
                .insert(topic_message.event().topic.clone(), update_status);
            available.available_latest = update_status;
            debug!(
                "entity: {} topic: {} status: {:?}",
                name,
                topic_message.event().topic.clone(),
                update_status
            );
        }
    }
}

pub(crate) fn try_render_available(
    avail_config: &MQTTAvailability,
    message: &TopicMessage,
) -> anyhow::Result<bool> {
    if let Some(config) = avail_config.topic.get(&message.topic) {
        let payload_available_str = config.payload_available();
        let payload_not_available_str = config.payload_not_available();
        let status_str = String::from_utf8(message.payload.to_vec())?;

        let match_value = match &config.value_template {
            Some(available_template) => {
                let mut env = Environment::new();
                env.add_template("available", &available_template)?;
                let template = env.get_template("available")?;
                let template_value =
                    if let Ok(state_json) = serde_json::from_str::<Value>(&status_str) {
                        template.render(context! { value_json => state_json })?
                    } else {
                        template.render(context! { value => status_str })?
                    };

                template_value
            }
            None => status_str,
        };

        return if payload_available_str == match_value {
            Ok(true)
        } else if payload_not_available_str == match_value {
            Ok(false)
        } else {
            Err(anyhow::anyhow!(
                "Invalid availability template: {}",
                match_value
            ))
        };
    }

    Ok(false)
}

#[test]
fn test_template() {
    let mut env = Environment::new();
    env.add_template("state", "{{ 'OFF' if 'no error' in value else 'ON' }}")
        .unwrap();

    let template = env.get_template("state").unwrap();
    let str = template.render(context! { value => "no error" }).unwrap();
    println!("str: {:?}", str);
}
