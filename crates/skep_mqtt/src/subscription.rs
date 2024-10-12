use crate::{
    discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload},
    entity::{MQTTAvailabilityConfiguration, MQTTRenderTemplate},
};
use bevy_core::Name;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::Trigger,
    prelude::{Added, Changed, Commands, Event, Query},
};
use bevy_hierarchy::BuildChildren;
use bevy_log::debug;
use bevy_mqtt::{rumqttc::QoS, SubscribeTopic, TopicMessage};
use bevy_reflect::{Map, Reflect};
use bevy_utils::{HashMap, HashSet};
use bytes::Bytes;
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use tera::{Context, Tera};

#[derive(Debug)]
pub struct EntitySubscription {
    topic: Option<String>,
    // message_callback: MessageCallbackType,
    should_subscribe: Option<bool>,
    // unsubscribe_callback: Option<Box<dyn Fn() + Send + Sync>>,
    qos: i32,
    encoding: String,
    entity_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Component, Reflect, Clone, PartialEq)]
pub struct MQTTStateSubscription {
    pub state_topic: String,
    pub value_template: Option<String>,
    pub qos: Option<i32>, // default 0
}

impl EntitySubscription {
    pub fn new(
        topic: Option<String>,
        // message_callback: MessageCallbackType,
        should_subscribe: Option<bool>,
        // unsubscribe_callback: Option<Box<dyn Fn() + Send + Sync>>,
        qos: i32,
        encoding: String,
        entity_id: Option<String>,
    ) -> Self {
        Self {
            topic,
            should_subscribe,
            qos,
            encoding,
            entity_id,
        }
    }

    pub async fn resubscribe_if_necessary(&mut self, other: Option<&EntitySubscription>) {
        if !self.should_resubscribe(other) {
            // if let Some(other) = other {
            //     self.unsubscribe_callback = other.unsubscribe_callback.clone();
            // }
            return;
        }

        // if let Some(other) = other {
        //     if let Some(unsubscribe_callback) = &other.unsubscribe_callback {
        //         unsubscribe_callback();
        //         debug_info::remove_subscription(
        //             &self.hass.lock().await,
        //             other.topic.as_deref().unwrap_or(""),
        //             other.entity_id.as_deref(),
        //         );
        //     }
        // }

        if self.topic.is_none() {
            return;
        }

        self.should_subscribe = Some(true);
    }

    pub async fn subscribe(&mut self) {
        if self.should_subscribe != Some(true) || self.topic.is_none() {
            return;
        }

        // self.unsubscribe_callback = Some(Box::new(async_subscribe_internal(
        //     self.hass.clone(),
        //     self.topic.clone().unwrap(),
        //     self.message_callback.clone(),
        //     self.qos,
        //     self.encoding.clone(),
        //     self.job_type.clone(),
        // )));
    }

    fn should_resubscribe(&self, other: Option<&EntitySubscription>) -> bool {
        if other.is_none() {
            return true;
        }

        let other = other.unwrap();
        self.topic != other.topic || self.qos != other.qos || self.encoding != other.encoding
    }
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

pub(crate) fn add_available_subscription(
    mut commands: Commands,
    mut q_available: Query<
        (
            Entity,
            &MQTTDiscoveryPayload,
            &mut MQTTAvailabilityConfiguration,
        ),
        Changed<MQTTDiscoveryPayload>,
    >,
) {
    for (entity, payload, mut avail_sub) in q_available.iter_mut() {
        if let Ok(available) = serde_json::from_value::<MQTTAvailabilityConfiguration>(Value::from(
            payload.payload.clone(),
        )) {
            if !available.availability_topic().is_empty() {
                let sub_topic = SubscribeTopic::new(available.availability_topic(), 0);
                let child_id = commands.spawn(sub_topic).id();
                commands
                    .entity(entity)
                    .add_child(child_id)
                    .observe(handle_available_value);
            }
            *avail_sub = available;
        }
    }
}

fn handle_state_value(
    topic_message: Trigger<TopicMessage>,
    q_state_sub: Query<(&MQTTStateSubscription, &Name)>,
) {
    if let Ok((state_sub, name)) = q_state_sub.get(topic_message.entity()) {
        let update_state = if let Some(value_template) = state_sub.value_template.as_ref() {
            let state_str = std::str::from_utf8(&topic_message.event().payload).unwrap();

            try_render_state(value_template, &state_str).unwrap_or_default()
        } else {
            let raw_value = std::str::from_utf8(&topic_message.event().payload)
                .unwrap()
                .to_string();
            // println!("raw_value: {:?}", raw_value);
            raw_value
        };
        if !update_state.is_empty() {
            debug!("{}: {}", name, update_state);
        }
    }
}

fn try_render_state(value_template: &String, state_str: &&str) -> anyhow::Result<String> {
    let mut env = Environment::new();
    env.add_template("state", value_template)?;

    let template = env.get_template("state")?;

    let template_value =
        if let Ok(state_json) = serde_json::from_str::<serde_json::Value>(&state_str) {
            let json_data = template.render(context! { value_json => state_json })?;
            json_data
        } else {
            let str = template.render(context! { value => state_str })?;

            str
        };

    // println!("template {} state: {}", value_template, template_value);
    Ok(template_value)
}

fn handle_available_value(
    topic_message: Trigger<TopicMessage>,
    q_avail: Query<(&MQTTAvailabilityConfiguration, &Name)>,
) {
    for ((available, name)) in q_avail.iter() {
        if topic_message.event().topic == available.availability_topic() {
            debug!(
                "available_update '{}' {:?}: {:?}",
                name,
                available.availability_topic(),
                topic_message.event().payload
            );
        }
    }
}

#[test]
fn test_template() {
    let mut env = Environment::new();
    env.add_template("state", "{{ 'OFF' if 'no error' in value else 'ON' }}")
        .unwrap();

    let template = env.get_template("state").unwrap();
    let str = template.render(context! { value => "no error" }).unwrap();
    println!("str: {:?}", str);
}
