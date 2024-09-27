use bevy_ecs::component::Component;

pub trait Integration: Component {
    fn name(&self) -> String;
}
