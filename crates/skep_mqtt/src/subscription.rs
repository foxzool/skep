use crate::{
    discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload},
    entity::{
        handle_available_value, handle_state_value, MQTTAvailability, MQTTAvailabilityConfiguration,
    },
};
use bevy_app::Update;
use bevy_core::Name;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::Trigger,
    prelude::{Added, Changed, Commands, Event, Query},
};
use bevy_hierarchy::{BuildChildren, Children, DespawnRecursiveExt, HierarchyQueryExt};
use bevy_log::debug;
use bevy_mqtt::{rumqttc::QoS, PacketCache, SubscribeTopic, TopicMessage};
use bevy_reflect::{Map, Reflect};
use bevy_utils::{HashMap, HashSet};
use bytes::Bytes;
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use skep_core::states::State;
use std::process::{Child, Command};
use tera::{Context, Tera};

#[derive(Debug, Serialize, Deserialize, Component, Reflect, Clone, PartialEq)]
pub struct MQTTStateSubscription {
    pub state_topic: String,
    pub value_template: Option<String>,
    pub qos: Option<u8>, // default 0
}

pub(crate) fn add_state_subscription(
    mut commands: Commands,
    mut q_discovery: Query<
        (Entity, &MQTTDiscoveryPayload, &mut MQTTStateSubscription),
        Changed<MQTTDiscoveryPayload>,
    >,
) {
    for (entity, payload, mut state_sub) in q_discovery.iter_mut() {
        if let Ok(sub) =
            serde_json::from_value::<MQTTStateSubscription>(Value::from(payload.payload.clone()))
        {
            let qos = sub.qos.unwrap_or(0);
            let state_topic = sub.state_topic.clone();
            *state_sub = sub;

            let sub_topic = SubscribeTopic::new(state_topic.clone(), qos);
            let child_id = commands.spawn(sub_topic).id();
            commands
                .entity(entity)
                .add_child(child_id)
                .observe(handle_state_value);
        }
    }
}

pub(crate) fn update_available_subscription(
    mut commands: Commands,
    q_child: Query<&mut SubscribeTopic>,
    mut q_available: Query<
        (
            Entity,
            &MQTTDiscoveryPayload,
            &mut MQTTAvailability,
            Option<&Children>,
        ),
        Changed<MQTTDiscoveryPayload>,
    >,
) {
    for (entity, payload, mut avail, opt_children) in q_available.iter_mut() {
        if let Ok(available_config) = serde_json::from_value::<MQTTAvailabilityConfiguration>(
            Value::from(payload.payload.clone()),
        ) {
            avail.update_from_config(available_config);

            if let Some(children) = opt_children {
                for avail_config in avail.topic.values() {
                    debug!("avail_config: {:?}", avail_config);
                    for child in children {
                        let child_id = *child;
                        let child_topic = q_child.get(child_id).unwrap().topic();
                        if child_topic == avail_config.topic {
                            continue;
                        }
                        let sub_topic = SubscribeTopic::new(child_topic, 0);
                        let child_id = commands.spawn((sub_topic, PacketCache::default())).id();
                        commands
                            .entity(entity)
                            .add_child(child_id)
                            .observe(handle_available_value);
                    }
                }
                // find need remove topic;
                for child in children {
                    let child_id = *child;
                    if let Ok(child_topic) = q_child.get(child_id) {
                        if !avail.topic.contains_key(child_topic.topic()) {
                            commands.entity(child_id).despawn_recursive();
                        }
                    }
                }
            } else {
                for avail_config in avail.topic.values() {
                    let sub_topic = SubscribeTopic::new(&avail_config.topic, 0);
                    let child_id = commands.spawn(sub_topic).id();
                    commands
                        .entity(entity)
                        .add_child(child_id)
                        .observe(handle_available_value);
                }
            }
        }
    }
}
