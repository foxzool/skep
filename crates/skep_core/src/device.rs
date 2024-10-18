use crate::helper::device_registry::DeviceInfo;
use bevy_app::{App, Plugin};
use bevy_core::Name;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::{ComponentHooks, StorageType},
    prelude::*,
    world::DeferredWorld,
};
use bevy_reflect::{Reflect, TypePath};
use bevy_utils::{HashMap, HashSet};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash};
use uuid::Uuid;

pub(crate) struct SkepDevicePlugin;

impl Plugin for SkepDevicePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Device>()
            .register_type::<DeviceInfo>()
            .init_resource::<DeviceResource>()
            // .add_systems(Update, device_create_or_update)
        ;
    }
}

#[derive(Debug, Reflect)]
pub struct Device {
    pub area_id: Option<String>,
    pub configuration_url: Option<String>,
    #[reflect(ignore)]
    pub created_at: chrono::DateTime<Utc>,
    #[reflect(ignore)]
    pub modified_at: chrono::DateTime<Utc>,
    pub connections: HashsetTupleString,
    pub disabled_by: Option<DeviceEntryDisabler>,
    pub entry_type: Option<DeviceEntryType>,
    pub hw_version: Option<String>,
    pub id: String,
    pub identifiers: HashsetTupleString,
    pub labels: HashSet<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub model_id: Option<String>,
    pub name_by_user: Option<String>,
    pub name: Option<String>,
    pub serial_number: Option<String>,
    pub suggested_area: Option<String>,
    pub sw_version: Option<String>,
    pub via_device_id: Option<String>,
}

impl Component for Device {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world: DeferredWorld, entity, _component_id| {
            let device = world.get::<Device>(entity).unwrap();
            let name = Name::new(device.name().to_string());
            let mut commands = world.commands();
            let mut binding = commands.entity(entity);
            binding.insert(name);
        });
    }
}

impl Default for Device {
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
            identifiers: Default::default(),
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

impl Device {
    pub fn update_from_device_info(&mut self, device_info: DeviceInfo) {
        self.configuration_url = device_info.configuration_url;
        self.connections = device_info.connections;
        self.entry_type = device_info.entry_type;
        self.hw_version = device_info.hw_version;
        self.identifiers = device_info.identifiers;
        self.labels = device_info.labels.unwrap_or_default();
        self.manufacturer = device_info.manufacturer;
        self.model = device_info.model;
        self.model_id = device_info.model_id;
        self.name = device_info.name;
        self.serial_number = device_info.serial_number;
        self.suggested_area = device_info.suggested_area;
        self.sw_version = device_info.sw_version;
        self.via_device_id = device_info.via_device_id;
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, Reflect, Copy)]
pub enum DeviceEntryType {
    #[default]
    Service,
}

#[derive(Debug, Deref, DerefMut, TypePath, Clone)]
pub struct TupleString(pub (String, String));

impl PartialEq<Self> for TupleString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Hash for TupleString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl Eq for TupleString {}

#[derive(Debug, Default, Clone, Reflect)]
pub struct HashsetTupleString(pub HashSet<TupleString>);

impl Display for HashsetTupleString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple_string = String::new();
        for tuple in &self.0 {
            tuple_string.push_str(&format!("'{}.{}' ", tuple.0 .0, tuple.0 .1));
        }
        write!(f, "{}", tuple_string)
    }
}

impl Hash for HashsetTupleString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for tuple in &self.0 {
            tuple.hash(state);
        }
    }
}

impl PartialEq<Self> for HashsetTupleString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for HashsetTupleString {}

#[derive(Debug, Resource, Default)]
pub struct DeviceResource {
    pub identifiers: HashMap<HashsetTupleString, Entity>,
    pub connections: HashMap<HashsetTupleString, Entity>,
}

impl DeviceResource {
    pub fn get_device(
        &self,
        identifiers: &HashsetTupleString,
        connections: &HashsetTupleString,
    ) -> Option<Entity> {
        if let Some(entity) = self.get_device_by_identifiers(identifiers) {
            return Some(entity);
        }
        self.get_device_by_connections(connections)
    }

    fn get_device_by_identifiers(&self, identifiers: &HashsetTupleString) -> Option<Entity> {
        if let Some(entity) = self.identifiers.get(identifiers) {
            return Some(*entity);
        }
        None
    }

    fn get_device_by_connections(&self, connections: &HashsetTupleString) -> Option<Entity> {
        if let Some(entity) = self.connections.get(connections) {
            return Some(*entity);
        }
        None
    }
}

// pub(crate) fn device_create_or_update(
//     mut commands: Commands,
//     parent_query: Query<&Parent>,
//     q_integration: Query<&Integration>,
//     mut q_devices: Query<&mut Device>,
//     mut q_device: Query<(Entity, &DeviceInfo), Added<DeviceInfo>>,
// ) {
//     for (entity, device_info) in q_device.iter_mut() {
//         let mut domain = "";
//
//         'fa: for ancestor in parent_query.iter_ancestors(entity) {
//             if let Ok(integration) = q_integration.get(ancestor) {
//                 domain = integration.domain.as_ref();
//                 break 'fa;
//             }
//         }
//         if domain.is_empty() {
//             continue;
//         }
//         // let mut new_device = true;
//         // 'fd: for mut device in q_devices.iter_mut() {
//         //     if device.identifiers == device_info.identifiers {
//         //         debug!("Device already exists, updating");
//         //         device.update_from_device_info(device_info.clone());
//         //         new_device = false;
//         //         break 'fd;
//         //     }
//         // }
//         //
//         // if new_device {
//         //     let mut device_entry = Device::default();
//         //     device_entry.update_from_device_info(device_info.clone());
//         //     commands.entity(entity).insert(device_entry);
//         // }
//     }
// }
