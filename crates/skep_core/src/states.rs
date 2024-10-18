use crate::context::Context;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::{change_detection::DetectChanges, prelude::*};
use bevy_reflect::Reflect;
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};

pub struct SkepStatePlugin;

impl Plugin for SkepStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_time);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct State {
    pub state: String,
    pub context: Context,
}

impl State {
    pub fn new(state: String) -> Self {
        Self {
            state,
            context: Default::default(),
        }
    }

    pub fn update(&mut self, new_state: impl ToString) {
        self.state = new_state.to_string();
    }
}

#[derive(Debug, Component, Default)]
pub struct StateUpdateTime {
    /// The last time the state was changed
    pub last_changed: Option<DateTime<Utc>>,
    /// The last time the state or state_attributes was reported
    pub last_reported: Option<DateTime<Utc>>,
    /// The last time the state or state_attributes was changed
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Component, Serialize, Deserialize, Reflect, Clone)]
pub struct StateAttributes {
    pub friendly_name: Option<String>,
    pub icon: Option<String>,
    pub entity_picture: Option<String>,
    pub assumed_state: Option<bool>,
    pub unit_of_measurement: Option<String>,
    pub attribution: Option<String>,
    pub device_class: Option<String>,
    pub supported_features: Option<i32>,
}

fn update_time(
    mut q_attr_changed: Query<(
        &mut StateUpdateTime,
        Ref<StateAttributes>,
        Option<Ref<State>>,
    )>,
) {
    for (mut update_time, attributes, opt_state) in q_attr_changed.iter_mut() {
        let now = Utc::now();
        if attributes.is_changed() {
            update_time.last_reported = Some(now);
            update_time.last_updated = Some(now);
        }

        if let Some(state) = opt_state {
            if state.is_changed() {
                update_time.last_reported = Some(now);
                update_time.last_changed = Some(now);
            }
        }
    }
}

#[test]
fn test_attributes() {
    let json = r#"{"~": "watermeter","unique_id": "watermeter-value","object_id": "watermeter_value","name": "Value","icon": "mdi:gauge","state_topic": "~/main/value","unit_of_meas": "m³","device_class": "water","state_class": "total_increasing","availability_topic": "~/connection","payload_available": "connected","payload_not_available": "connection lost","device": {"identifiers": ["watermeter"],"name": "watermeter","model": "Meter Digitizer","manufacturer": "AI on the Edge Device","sw_version": "v15.7.0","configuration_url": "http://192.168.1.41"}}"#;

    let attributes: StateAttributes = serde_json::from_str(json).unwrap();
    assert_eq!(attributes.friendly_name, None);
    assert_eq!(attributes.icon, Some("mdi:gauge".to_string()));
    assert_eq!(attributes.entity_picture, None);
    assert_eq!(attributes.assumed_state, None);
    assert_eq!(attributes.unit_of_measurement, None);
    assert_eq!(attributes.attribution, None);
    assert_eq!(attributes.device_class, Some("water".to_string()));
    assert_eq!(attributes.supported_features, None);
}
