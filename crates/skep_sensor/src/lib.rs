use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chrono::{DateTime, Utc};
use log::debug;
use std::str::FromStr;

mod constant;
pub use constant::*;
use skep_core::{
    device::DeviceEntry,
    typing::{SetupConfigEvent, ValueType},
};

pub struct SkepSensorPlugin;

impl Plugin for SkepSensorPlugin {
    fn build(&self, app: &mut App) {
        app.observe(create_or_update);
    }
}

#[derive(Debug, Component)]
pub struct Sensor {
    pub device_class: Option<SensorDeviceClass>,
    pub last_reset: Option<DateTime<Utc>>,
    pub native_unit_of_measurement: Option<String>,
    pub native_value: Option<ValueType>,
    pub options: Option<Vec<String>>,
    pub state_class: Option<String>,
    pub suggested_display_precision: Option<i32>,
    pub suggested_unit_of_measurement: Option<String>,
    pub unit_of_measurement: Option<String>,
}

impl Sensor {
    pub fn from_config(event: SetupConfigEvent) -> Sensor {
        match SensorDeviceClass::from_str(&event.component) {
            Ok(device_class) => Sensor {
                device_class: Some(device_class.clone()),
                last_reset: None,
                native_unit_of_measurement: device_class.unit_of_measurement(),
                native_value: None,
                options: None,
                state_class: None,
                suggested_display_precision: None,
                suggested_unit_of_measurement: None,
                unit_of_measurement: None,
            },
            Err(_) => Sensor {
                device_class: None,
                last_reset: None,
                native_unit_of_measurement: None,
                native_value: None,
                options: None,
                state_class: None,
                suggested_display_precision: None,
                suggested_unit_of_measurement: None,
                unit_of_measurement: None,
            },
        }
    }
}

fn create_or_update(trigger: Trigger<SetupConfigEvent>, device_query: Query<&DeviceEntry>) {
    debug!("sensor create_or_update {:?}", trigger.event());
    for device in device_query.iter() {
        println!("device: {}", device.name());
    }
}
