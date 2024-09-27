use bevy_app::{App, Plugin};
use bevy_ecs::component::Component;
use chrono::{DateTime, Utc};

mod constant;
pub use constant::*;
use skep_core::typing::ValueType;

pub struct SkepSensorPlugin;

impl Plugin for SkepSensorPlugin {
    fn build(&self, _app: &mut App) {}
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
