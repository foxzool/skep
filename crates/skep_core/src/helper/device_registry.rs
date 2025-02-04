use crate::device::{DeviceEntryType, HashsetTupleString, TupleString};
use bevy_reflect::Reflect;
use bevy_utils::{tracing::debug, HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::StringOrVecToVec;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Default, Clone, Reflect)]
pub struct DeviceInfo {
    pub configuration_url: Option<String>,
    pub connections: HashsetTupleString,
    pub default_manufacturer: Option<String>,
    pub default_model: Option<String>,
    pub default_name: Option<String>,
    pub entry_type: Option<DeviceEntryType>,
    pub identifiers: HashsetTupleString,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<String>,
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
    pub fn update_from(&mut self, new: DeviceInfo) {
        self.configuration_url = new.configuration_url.or(self.configuration_url.clone());
        self.connections = new.connections;
        self.default_manufacturer = new.default_manufacturer;
        self.default_model = new.default_model;
        self.default_name = new.default_name;
        self.entry_type = new.entry_type.or(self.entry_type);
        self.identifiers = new.identifiers;
        self.entry_type = new.entry_type;
        self.hw_version = new.hw_version;
        self.labels = new.labels.or(self.labels.clone());
        self.manufacturer = new.manufacturer;
        self.model = new.model;
        self.model_id = new.model_id;
        self.name = new.name;
        self.serial_number = new.serial_number;
        self.suggested_area = new.suggested_area;
        self.sw_version = new.sw_version;
        self.via_device_id = new.via_device_id;
    }
    pub fn from_config(domain: &str, config: DeviceSpec) -> DeviceInfo {
        DeviceInfo {
            identifiers: Self::parse_identifiers(domain, config.identifiers),
            manufacturer: config.manufacturer,
            model: config.model,
            model_id: config.model_id,
            name: config.name,
            serial_number: config.serial_number,
            suggested_area: config.suggested_area,
            sw_version: config.sw_version,
            hw_version: config.hw_version,
            labels: config.labels,
            translation_key: config.translation_key,
            translation_placeholders: config.translation_placeholders,
            connections: Self::parse_connections(config.connections.unwrap_or_default()),
            configuration_url: config.configuration_url,
            default_manufacturer: config.default_manufacturer,
            default_model: config.default_model,
            default_name: config.default_name,
            entry_type: config.entry_type,
            via_device_id: config.via_device_id,
        }
    }

    fn parse_identifiers(domain: &str, identifiers: Vec<String>) -> HashsetTupleString {
        HashsetTupleString(
            identifiers
                .into_iter()
                .map(|v| TupleString((domain.to_string(), v)))
                .collect(),
        )
    }

    fn parse_connections(connections: Vec<Vec<String>>) -> HashsetTupleString {
        HashsetTupleString(
            connections
                .into_iter()
                .filter(|v| v.len() >= 2)
                .map(|v| TupleString((v[0].to_string(), v[1].to_string())))
                .collect(),
        )
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub configuration_url: Option<String>,
    pub connections: Option<Vec<Vec<String>>>,
    pub default_manufacturer: Option<String>,
    pub default_model: Option<String>,
    pub default_name: Option<String>,
    pub entry_type: Option<DeviceEntryType>,
    #[serde(deserialize_with = "identifiers_parser")]
    pub identifiers: Vec<String>,
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

fn identifiers_parser<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de> + 'static,
    <T as FromStr>::Err: std::fmt::Display,
{
    StringOrVecToVec::default().into_deserializer()(deserializer)
}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.identifiers == other.identifiers || self.connections == other.connections
    }
}
