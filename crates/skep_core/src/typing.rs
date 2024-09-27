use bevy_ecs::event::Event;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{Map, Value};

#[derive(Debug)]
pub enum ValueType {
    String(String),
    Float(f64),
    Int(i64),
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
}

pub type ConfigType = Map<String, Value>;

#[derive(Event, Debug, Clone)]
pub struct SetupConfigEvent {
    pub component: String,
    pub object_id: String,
    pub payload: Value,
}
