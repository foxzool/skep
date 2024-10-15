use crate::{
    discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload},
    entity::{MQTTAvailability, MQTTAvailabilityConfiguration},
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
use bevy_mqtt::{rumqttc::QoS, SubscribeTopic, TopicMessage};
use bevy_reflect::{Map, Reflect};
use bevy_utils::{HashMap, HashSet};
use bytes::Bytes;
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Child, Command};
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
                    for child in children {
                        let child_id = *child;
                        let child_topic = q_child.get(child_id).unwrap().topic();
                        if child_topic == avail_config.topic {
                            continue;
                        }
                        let sub_topic = SubscribeTopic::new(child_topic, 0);
                        let child_id = commands.spawn(sub_topic).id();
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

fn handle_state_value(
    topic_message: Trigger<TopicMessage>,
    q_state_sub: Query<(&MQTTStateSubscription, &Name)>,
) {
    if let Ok((state_sub, name)) = q_state_sub.get(topic_message.entity()) {
        if topic_message.event().topic == state_sub.state_topic {
            let update_state =
                try_render_template(&state_sub.value_template, &topic_message.event().payload)
                    .unwrap_or_default();
            if !update_state.is_empty() {
                debug!("{}: {}", name, update_state);
            }
        }
    }
}

fn try_render_template(
    value_template: &Option<String>,
    value_bytes: &[u8],
) -> anyhow::Result<String> {
    if let Some(value_template) = value_template {
        let mut env = Environment::new();
        env.add_template("value", &value_template)?;
        let template = env.get_template("value")?;

        let template_value = if let Ok(state_json) = serde_json::from_slice::<Value>(value_bytes) {
            template.render(context! { value_json => state_json })?
        } else {
            template.render(context! { value => value_bytes })?
        };

        Ok(template_value)
    } else {
        Ok(std::str::from_utf8(value_bytes)?.to_string())
    }
}

fn handle_available_value(
    topic_message: Trigger<TopicMessage>,
    mut q_avail: Query<(&mut MQTTAvailability, &Name)>,
) {
    if let Ok((mut available, name)) = q_avail.get_mut(topic_message.entity()) {
        let update_status = try_render_available(&available, &topic_message.event()).ok();
        if let Some(update_status) = update_status {
            available
                .avail_topics
                .insert(topic_message.event().topic.clone(), update_status);
            available.available_latest = update_status;
            debug!(
                "{} {} status: {:?}",
                name,
                topic_message.event().topic.clone(),
                update_status
            );
        }
    }
}

fn try_render_available(
    avail_config: &MQTTAvailability,
    message: &TopicMessage,
) -> anyhow::Result<bool> {
    if let Some(config) = avail_config.topic.get(&message.topic) {
        let payload_available_str = config.payload_available();
        let payload_not_available_str = config.payload_not_available();
        let status_str = String::from_utf8(message.payload.to_vec())?;

        let match_value = match &config.value_template {
            Some(available_template) => {
                let mut env = Environment::new();
                env.add_template("available", &available_template)?;
                let template = env.get_template("available")?;
                let template_value =
                    if let Ok(state_json) = serde_json::from_str::<Value>(&status_str) {
                        template.render(context! { value_json => state_json })?
                    } else {
                        template.render(context! { value => status_str })?
                    };

                template_value
            }
            None => status_str,
        };

        return if payload_available_str == match_value {
            Ok(true)
        } else if payload_not_available_str == match_value {
            Ok(false)
        } else {
            Err(anyhow::anyhow!(
                "Invalid availability template: {}",
                match_value
            ))
        };
    }

    Ok(false)
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
