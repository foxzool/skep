use crate::{
    constants::{CONF_ENABLED_BY_DEFAULT, CONF_OBJECT_ID},
    sensor::MqttSensorComponent,
    subscription::EntitySubscription,
    DiscoveryInfoType,
};
use bevy_ecs::prelude::ResMut;
use bevy_utils::HashMap;
use serde_json::Value;
use skep_core::{
    config_entry::ConfigEntry,
    constants::{
        EntityCategory, CONF_DEVICE, CONF_ENTITY_CATEGORY, CONF_ICON, CONF_NAME, CONF_UNIQUE_ID,
    },
    helper::entity::{SkepEntity, SkepEntityComponent},
    typing::ConfigType,
    SkepResource,
};
use std::str::FromStr;

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
    fn init(&mut self, config: ConfigType);
    fn attributes_sub_state(&self) -> &HashMap<String, EntitySubscription>;
    fn attributes_config(&self) -> &ConfigType;
}

pub trait MqttAvailability: SkepEntity {
    fn init(config: &ConfigType) -> Self;
    fn availability_setup_from_config(&mut self, config: &ConfigType);
}

pub trait MqttDiscoveryUpdateMixin: SkepEntity {
    fn init(
        discovery_data: Option<DiscoveryInfoType>,
        // discovery_update: Option<
        //     // Box<dyn Fn(MQTTDiscoveryPayload) -> Pin<Box<dyn Future<Output = ()>>>>,
        // >,
    ) -> Self;

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
