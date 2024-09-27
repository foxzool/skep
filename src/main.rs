use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use skep_core::SkepCorePlugin;
use skep_mqtt::SkepMqttPlugin;
use skep_sensor::SkepSensorPlugin;
use std::time::Duration;

fn main() {
    let mut app = App::new();
    if cfg!(feature = "gui") {
        app.add_plugins(DefaultPlugins)
            .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
            .add_systems(Startup, setup)
    } else {
        App::new().add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
            LogPlugin::default(),
        ))
    };

    app.add_plugins(SkepCorePlugin)
        .add_plugins(SkepSensorPlugin)
        .add_plugins(SkepMqttPlugin)
        .run();
}

#[allow(dead_code)]
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
