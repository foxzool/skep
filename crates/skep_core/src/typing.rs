use bevy_ecs::event::Event;
use chrono::{DateTime, NaiveDate, Utc};
use either::Either;
use serde_json::{Map, Value};

pub type StateType = Option<Either<String, Either<i32, f64>>>;

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
pub struct SetupConfigEntry {
    pub component: String,
    pub object_id: String,
    pub payload: Value,
}
