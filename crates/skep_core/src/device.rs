use crate::{integration::Integration, platform::Platform, typing::ConfigType};
use bevy_app::{App, Plugin};
use bevy_core::Name;
use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use bevy_utils::{tracing::debug, HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use uuid::Uuid;

pub(crate) struct SkepDevicePlugin;

impl Plugin for SkepDevicePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DeviceEntry>()
            .observe(device_create_or_update);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct DeviceEntry {
    pub area_id: Option<String>,
    pub configuration_url: Option<String>,
    #[reflect(ignore)]
    pub created_at: chrono::DateTime<Utc>,
    pub connections: HashSet<(String, String)>,
    pub disabled_by: Option<DeviceEntryDisabler>,
    pub entry_type: Option<DeviceEntryType>,
    pub hw_version: Option<String>,
    pub id: String,
    pub identifiers: HashSet<(String, String)>,
    pub labels: HashSet<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<String>,
    #[reflect(ignore)]
    pub modified_at: chrono::DateTime<Utc>,
    pub name_by_user: Option<String>,
    pub name: Option<String>,
    pub serial_number: Option<String>,
    pub suggested_area: Option<String>,
    pub sw_version: Option<String>,
    pub via_device_id: Option<String>,
}

impl Default for DeviceEntry {
    fn default() -> Self {
        Self {
            area_id: None,
            configuration_url: None,
            created_at: chrono::DateTime::from(Utc::now()),
            connections: Default::default(),
            disabled_by: None,
            entry_type: None,
            hw_version: None,
            id: Uuid::new_v4().to_string(),
            name: None,
            serial_number: None,
            suggested_area: None,
            sw_version: None,
            identifiers: HashSet::new(),
            labels: Default::default(),
            manufacturer: None,
            model: None,
            model_id: None,
            modified_at: Default::default(),
            name_by_user: None,
            via_device_id: None,
        }
    }
}

impl DeviceEntry {
    pub fn update_from_device_info(&mut self, device_info: DeviceInfo) {
        self.configuration_url = device_info.configuration_url;
        self.connections = device_info.connections.unwrap_or_default();
        self.entry_type = device_info.entry_type;
        self.hw_version = device_info.hw_version;
        self.identifiers = device_info.identifiers.unwrap_or_default();
        self.labels = device_info.labels.unwrap_or_default();
        self.manufacturer = device_info.manufacturer;
        self.model = device_info.model;
        self.model_id = device_info.model_id;
        self.modified_at = device_info.modified_at.unwrap_or_default();
        self.name = device_info.name;
        self.serial_number = device_info.serial_number;
        self.suggested_area = device_info.suggested_area;
        self.sw_version = device_info.sw_version;
        self.via_device_id = device_info.via_device_id;
    }
    pub fn device_info(&self) -> DeviceInfo {
        DeviceInfo {
            configuration_url: self.configuration_url.clone(),
            connections: Some(self.connections.clone()),
            default_manufacturer: self.manufacturer.clone(),
            default_model: self.model.clone(),
            default_name: self.name.clone(),
            entry_type: self.entry_type.clone(),
            identifiers: Some(self.identifiers.clone()),
            manufacturer: self.manufacturer.clone(),
            model: self.model.clone(),
            model_id: self.model_id.clone(),
            modified_at: Some(self.modified_at),
            name: self.name.clone(),
            serial_number: self.serial_number.clone(),
            suggested_area: self.suggested_area.clone(),
            sw_version: self.sw_version.clone(),
            hw_version: self.hw_version.clone(),
            labels: Some(self.labels.clone()),
            translation_key: None,
            translation_placeholders: None,
            via_device_id: self.via_device_id.clone(),
        }
    }

    pub fn name(&self) -> &str {
        if let Some(name_by_user) = self.name_by_user.as_deref() {
            name_by_user
        } else {
            self.name.as_deref().unwrap_or_default()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Reflect)]
pub enum DeviceEntryDisabler {
    ConfigEntry,
    Integration,
    User,
}

#[derive(Debug, Default, Clone, Deserialize, Reflect)]
pub enum DeviceEntryType {
    #[default]
    Service,
}

#[derive(Debug, Default, Clone, Deserialize, Event)]
pub struct DeviceInfo {
    pub configuration_url: Option<String>,
    pub connections: Option<HashSet<(String, String)>>,
    pub default_manufacturer: Option<String>,
    pub default_model: Option<String>,
    pub default_name: Option<String>,
    pub entry_type: Option<DeviceEntryType>,
    #[serde(skip)]
    pub identifiers: Option<HashSet<(String, String)>>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<String>,
    pub modified_at: Option<chrono::DateTime<Utc>>,
    pub name: Option<String>,
    pub serial_number: Option<String>,
    pub suggested_area: Option<String>,
    pub sw_version: Option<String>,
    pub hw_version: Option<String>,
    pub labels: Option<HashSet<String>>,
    pub translation_key: Option<String>,
    pub translation_placeholders: Option<HashMap<String, String>>,
    pub via_device_id: Option<String>,
}

impl DeviceInfo {
    pub fn from_domain_config(domain: &str, config_type: ConfigType) -> anyhow::Result<DeviceInfo> {
        let mut device_info: DeviceInfo = serde_json::from_value(Value::from(config_type.clone()))?;
        if let Some(identifiers) = config_type.get("identifiers") {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum StringOrSet {
                String(String),
                HashSet(HashSet<String>),
                Null,
            }

            let i = match StringOrSet::deserialize(identifiers)? {
                StringOrSet::String(s) => {
                    let mut set = HashSet::new();
                    set.insert((domain.to_string(), s));
                    Some(set)
                }
                StringOrSet::HashSet(set) => {
                    if set.is_empty() {
                        None
                    } else {
                        Some(set.into_iter().map(|s| (domain.to_string(), s)).collect())
                    }
                }
                StringOrSet::Null => None,
            };

            device_info.identifiers = i;
        };

        Ok(device_info)
    }
}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.identifiers == other.identifiers || self.connections == other.connections
    }
}

fn deserialize_identifiers<'de, D>(deserializer: D) -> Result<Option<HashSet<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrSet {
        String(String),
        HashSet(HashSet<String>),
        Null,
    }

    match StringOrSet::deserialize(deserializer)? {
        StringOrSet::String(s) => {
            let mut set = HashSet::new();
            set.insert(s);
            Ok(Some(set))
        }
        StringOrSet::HashSet(set) => {
            if set.is_empty() {
                Ok(None)
            } else {
                Ok(Some(set))
            }
        }
        StringOrSet::Null => Ok(None),
    }
}

impl DeviceInfo {
    /// when connections or identifiers any one has set and has value, return true
    pub fn is_valid(&self) -> bool {
        match (&self.connections, &self.identifiers) {
            (Some(connections), _) if !connections.is_empty() => true,
            (_, Some(identifiers)) if !identifiers.is_empty() => true,
            _ => false,
        }
    }
}

impl From<DeviceInfo> for DeviceEntry {
    fn from(device_info: DeviceInfo) -> Self {
        Self {
            area_id: None,
            configuration_url: device_info.configuration_url,
            created_at: chrono::DateTime::from(Utc::now()),
            connections: device_info.connections.unwrap_or_default(),
            disabled_by: None,
            entry_type: device_info.entry_type,
            hw_version: device_info.hw_version,
            id: Uuid::new_v4().to_string(),
            identifiers: device_info.identifiers.unwrap_or_default(),
            labels: device_info.labels.unwrap_or_default(),
            manufacturer: device_info.manufacturer,
            model: device_info.model,
            model_id: device_info.model_id,
            modified_at: device_info.modified_at.unwrap_or_default(),
            name_by_user: None,
            name: device_info.name,
            serial_number: device_info.serial_number,
            suggested_area: device_info.suggested_area,
            sw_version: device_info.sw_version,
            via_device_id: device_info.via_device_id,
        }
    }
}

pub(crate) fn device_create_or_update(
    trigger: Trigger<DeviceInfo>,
    mut commands: Commands,
    platform_query: Query<(&Integration, &Platform), Without<DeviceEntry>>,
    mut full_device_query: Query<(&Integration, &Platform, &mut DeviceEntry)>,
    mut raw_device_query: Query<&mut DeviceEntry, (Without<Integration>, Without<Platform>)>,
) {
    let device_info = trigger.event().clone();

    let device_entry = DeviceEntry::from(device_info.clone());
    debug!("spawn device_entry {:?}", device_entry);
    if let Ok((parent_integration, parent_platform)) = platform_query.get(trigger.entity()) {
        // has integration and platform
        for (integration, platform, mut device_entry) in full_device_query.iter_mut() {
            if parent_integration == integration
                && parent_platform == platform
                && device_entry.device_info() == device_info
            {
                device_entry.update_from_device_info(device_info);
                return;
            }
        }

        commands.spawn((
            Name::new(device_entry.name().to_string()),
            device_entry,
            parent_platform.clone(),
            parent_integration.clone(),
        ));
    } else {
        // no integration and platform
        for mut device_entry in raw_device_query.iter_mut() {
            if device_entry.device_info() == device_info {
                device_entry.update_from_device_info(device_info);
                return;
            }
        }

        commands.spawn((Name::new(device_entry.name().to_string()), device_entry));
    }
}
