use bevy_app::{App, Plugin};
use bevy_ecs::observer::Trigger;
use skep_core::typing::SetupConfigEntry;

pub struct MqttBinarySensorPlugin;

impl Plugin for MqttBinarySensorPlugin {
    fn build(&self, app: &mut App) {
        app.observe(on_setup_entry);
    }
}

const DOMAIN: &str = "binary_sensor";

fn on_setup_entry(trigger: Trigger<SetupConfigEntry>) {
    if trigger.event().component == DOMAIN {
        println!("Setup binary sensor");
    }
}
