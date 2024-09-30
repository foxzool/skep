use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hierarchy::BuildChildren;
use bevy_reflect::Reflect;
use chrono::{DateTime, Utc};
use log::debug;
use std::str::FromStr;

mod constant;
pub use constant::*;
use skep_core::{
    device::DeviceEntry,
    typing::{SetupConfigEntry, ValueType},
};

pub struct SkepSensorPlugin;

impl Plugin for SkepSensorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sensor>().observe(create_or_update);
    }
}

#[derive(Debug, Component, Default, Reflect)]
pub struct Sensor {
    pub device_class: Option<SensorDeviceClass>,
    #[reflect(ignore)]
    pub last_reset: Option<DateTime<Utc>>,
    pub native_unit_of_measurement: Option<String>,
    #[reflect(ignore)]
    pub native_value: Option<ValueType>,
    pub options: Option<Vec<String>>,
    pub state_class: Option<String>,
    pub suggested_display_precision: Option<i32>,
    pub suggested_unit_of_measurement: Option<String>,
    pub unit_of_measurement: Option<String>,
}

impl Sensor {
    pub fn from_config(event: SetupConfigEntry) -> anyhow::Result<Sensor> {
        if event.component != "sensor" {
            return Err(anyhow::anyhow!("Invalid component"));
        }
        let mut sensor = Sensor::default();
        // sensor.device_class =
        // SensorDeviceClass::from_str(&event.payload.get("device_class")?).ok();
        Ok(sensor)
    }
}

fn create_or_update(
    trigger: Trigger<SetupConfigEntry>,
    device_query: Query<&DeviceEntry>,
    mut commands: Commands,
) {
    if let Ok(sensor) = Sensor::from_config(trigger.event().clone()) {
        // debug!("sensor create_or_update {:?}", sensor);
        // let sensor_entity = commands.spawn_empty().insert(sensor).id();
        // if let Ok(_device) = device_query.get(trigger.entity()) {
        //     commands.entity(trigger.entity()).add_child(sensor_entity);
        // }
    }
}
