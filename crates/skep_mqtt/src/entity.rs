use crate::{
    constants::{
        CONF_AVAILABILITY, CONF_AVAILABILITY_TEMPLATE, CONF_AVAILABILITY_TOPIC,
        CONF_ENABLED_BY_DEFAULT, CONF_OBJECT_ID, CONF_PAYLOAD_AVAILABLE,
        CONF_PAYLOAD_NOT_AVAILABLE, CONF_TOPIC,
    },
    discovery::MQTTDiscoveryPayload,
    sensor::MqttSensorComponent,
    subscription::EntitySubscription,
    DiscoveryInfoType,
};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    prelude::{In, ResMut, System},
};
use bevy_reflect::{Reflect, ReflectDeserialize, ReflectSerialize};
use bevy_utils::HashMap;
use minijinja::{Environment, Template};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{
        EntityCategory, CONF_DEVICE, CONF_ENTITY_CATEGORY, CONF_ICON, CONF_NAME, CONF_UNIQUE_ID,
        CONF_VALUE_TEMPLATE,
    },
    helper::entity::{SkepEntity, SkepEntityComponent},
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
    sub_state: HashMap<String, EntitySubscription>,
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

pub trait MqttAttributesMixin: SkepEntity {
    fn init_attributes(&mut self, config: ConfigType);
    fn attributes_sub_state(&self) -> &HashMap<String, EntitySubscription>;
    fn set_attributes_sub_state(&mut self, sub_state: HashMap<String, EntitySubscription>);
    fn attributes_config(&self) -> &ConfigType;
    fn set_attributes_config(&mut self, config: ConfigType);
}

pub trait MqttAvailabilityMixin: SkepEntity {
    fn init_availability(&mut self, config: &ConfigType);
    fn set_available_latest(&mut self, available: bool);
    fn availability_setup_from_config(&mut self, config: &ConfigType) {
        let mut avail_topics = HashMap::new();

        if let Some(topic) = config.get(CONF_AVAILABILITY_TOPIC) {
            avail_topics.insert(
                topic.to_string(),
                HashMap::from([
                    (
                        CONF_PAYLOAD_AVAILABLE.to_string(),
                        config[CONF_PAYLOAD_AVAILABLE].clone(),
                    ),
                    (
                        CONF_PAYLOAD_NOT_AVAILABLE.to_string(),
                        config[CONF_PAYLOAD_NOT_AVAILABLE].clone(),
                    ),
                    (
                        CONF_AVAILABILITY_TEMPLATE.to_string(),
                        config
                            .get(CONF_AVAILABILITY_TEMPLATE)
                            .cloned()
                            .unwrap_or_default(),
                    ),
                ]),
            );
        }

        if let Some(availability) = config.get(CONF_AVAILABILITY) {
            if let Value::Array(avails) = availability {
                for avail_value in avails {
                    if let Value::Object(avail) = avail_value {
                        if let Some(topic) = avail.get(CONF_TOPIC) {
                            if let Value::String(topic_str) = topic {
                                avail_topics.insert(
                                    topic_str.to_string(),
                                    [
                                        (
                                            CONF_PAYLOAD_AVAILABLE.to_string(),
                                            avail[CONF_PAYLOAD_AVAILABLE].clone(),
                                        ),
                                        (
                                            CONF_PAYLOAD_NOT_AVAILABLE.to_string(),
                                            avail[CONF_PAYLOAD_NOT_AVAILABLE].clone(),
                                        ),
                                        (
                                            CONF_VALUE_TEMPLATE.to_string(),
                                            match avail.get(CONF_VALUE_TEMPLATE) {
                                                Some(template) => template.clone(),
                                                None => Value::Null,
                                            },
                                        ),
                                    ]
                                    .iter()
                                    .cloned()
                                    .collect(),
                                );
                            }
                        }
                    }
                }
            }
        }

        for avail_topic_conf in avail_topics.values_mut() {
            if let Some(_template) = avail_topic_conf.get_mut(CONF_AVAILABILITY_TEMPLATE) {

                // TODO
                // if !template.is_empty() {
                //     *template = MqttValueTemplate::new(template.clone(), self)
                //         .async_render_with_possible_json_value();
                // }
            }
        }

        self.set_avail_topics(avail_topics);
        self.set_avail_config(config.clone());
    }

    fn set_avail_config(&mut self, config: ConfigType);

    fn set_avail_topics(&mut self, avail_topics: HashMap<String, HashMap<String, Value>>);
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

pub trait MqttEntity:
    MqttAttributesMixin + MqttAttributesMixin + MqttDiscoveryUpdateMixin + MqttEntityDeviceInfo
{
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MQTTAvailabilityConfiguration {
    pub payload_available: Option<String>,
    pub payload_not_available: Option<String>,
    pub availability: Option<Vec<AvailabilityConfig>>,
    pub availability_topic: Option<String>,
    pub availability_template: Option<String>,
    pub availability_mode: Option<MQTTAvailabilityMode>,
}

#[derive(Debug, Serialize, Deserialize, Reflect)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Reflect)]
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
