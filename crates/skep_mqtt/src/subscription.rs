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
