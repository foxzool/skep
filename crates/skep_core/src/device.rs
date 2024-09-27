use crate::typing::SetupConfig;
use bevy_ecs::prelude::*;
use bevy_hierarchy::BuildChildren;
use bevy_utils::{HashMap, HashSet};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Component)]
pub struct DeviceEntry {
    pub area_id: Option<String>,
    pub configuration_url: Option<String>,
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
    pub fn from_config(event: SetupConfig) -> Option<DeviceEntry> {
        if let Some(device_value) = event.payload.get("device") {
            let mut device_entry = DeviceEntry::default();
            if let Some(identifiers) = device_value.get("identifiers") {
                if let Some(identifiers) = identifiers.as_array() {
                    for identifier in identifiers {
                        if let Some(identifier) = identifier.as_str() {
                            device_entry
                                .identifiers
                                .insert((identifier.to_string(), "".to_string()));
                        }
                    }
                }
            }
            if let Some(name) = device_value.get("name") {
                device_entry.name = Some(name.as_str().unwrap_or_default().to_string());
            }

            Some(device_entry)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeviceEntryDisabler {
    ConfigEntry,
    Integration,
    User,
}

#[derive(Debug, Default, Clone, Deserialize)]
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

pub(crate) fn device_create_or_update(trigger: Trigger<DeviceInfo>, mut commands: Commands) {
    println!("{:?}, {:?}", trigger.entity(), trigger.event());
    let device = commands
        .spawn(DeviceEntry::from(trigger.event().clone()))
        .id();
    commands.entity(trigger.entity()).add_child(device);
}

#[test]
fn test_device_info() {
    let device_json = json!(
        {
            "identifiers":[
               ("sensor",  "01ad")
            ],
            "name":"Garden"
        }
    );

    let device_info: DeviceInfo = serde_json::from_value(device_json).unwrap();
    assert!(device_info.is_valid())
}
