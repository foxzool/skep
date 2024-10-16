use crate::domain::Domain;
use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

#[derive(Debug, Component, Clone, PartialEq, Reflect)]
pub struct Integration {
    pub name: String,
    pub domain: String,
}
