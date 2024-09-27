use bevy_ecs::prelude::Component;
use bevy_reflect::Reflect;

#[derive(Debug, Component, Reflect)]
pub struct Platform {
    pub name: String,
}

impl Platform {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
