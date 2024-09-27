use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

#[derive(Component, Debug, Reflect)]
pub struct Domain {
    pub name: String,
}
