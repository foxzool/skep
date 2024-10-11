use crate::discovery::{MQTTDiscoveryHash, MQTTDiscoveryPayload};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::Trigger,
    prelude::{Added, Changed, Commands, Query},
};
use bevy_hierarchy::BuildChildren;
use bevy_log::debug;
use bevy_mqtt::{rumqttc::QoS, SubscribeTopic, TopicMessage};
use bevy_reflect::Map;
use bevy_utils::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

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

#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq, Clone)]
pub struct MqttStateSubscription {
    pub state_topic: String,
    pub qos: Option<i32>,
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

#[derive(Component, Debug, Default)]
pub struct MqttEntitySubscriptionManager {
    pub state_subs: HashMap<String, i32>,
}

pub(crate) fn add_state_subscription(
    mut commands: Commands,
    mut q_discovery: Query<
        (
            Entity,
            &MQTTDiscoveryPayload,
            &mut MqttEntitySubscriptionManager,
        ),
        Changed<MQTTDiscoveryPayload>,
    >,
) {
    for (entity, payload, mut sub_manager) in q_discovery.iter_mut() {
        if let Ok(sub) =
            serde_json::from_value::<MqttStateSubscription>(Value::from(payload.payload.clone()))
        {
            debug!("subscription changed {:#?}", sub);

            let qos = sub.qos.unwrap_or(0);
            let state_topic = sub.state_topic.clone();
            let old_qos = sub_manager.state_subs.get(&state_topic).copied();

            if old_qos == Some(qos) {
                continue;
            }

            sub_manager.state_subs.insert(state_topic.clone(), qos);
            let sub_topic = SubscribeTopic::new(state_topic.clone(), qos);
            let child_id = commands
                .spawn(sub_topic)
                .observe(move |topic_message: Trigger<TopicMessage>| {
                    println!(
                        "{} received: {:?}",
                        state_topic,
                        topic_message.event().payload
                    );
                })
                .id();
            commands.entity(entity).add_child(child_id);
        }
    }
}
