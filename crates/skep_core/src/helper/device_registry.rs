use crate::{device::DeviceEntryType, typing::ConfigType};
use bevy_ecs::prelude::Component;
use bevy_reflect::Reflect;
use bevy_utils::{HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Component, Reflect)]
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
    #[reflect(ignore)]
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
