use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

#[derive(Debug, Component, Clone, PartialEq, Reflect)]
pub struct Integration {
    pub name: String,
}

impl Integration {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
