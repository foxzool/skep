use bevy_app::{App, Plugin, Update};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    prelude::{Commands, Component, Entity, Query, Res},
    world::CommandQueue,
};
use bevy_time::{Time, Timer, TimerMode};

pub struct SkepCoreEventPlugin;

impl Plugin for SkepCoreEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_delayed_action);
    }
}

pub struct DelayedAction {
    pub timer: Timer,
    pub action: CommandQueue,
}

impl DelayedAction {
    pub fn new(secs_f32: f32, action: CommandQueue) -> Self {
        Self {
            timer: Timer::from_seconds(secs_f32, TimerMode::Once),
            action,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct DelayedActions(pub Vec<DelayedAction>);

fn check_delayed_action(
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedActions)>,
    mut commands: Commands,
) {
    for (entity, mut actions) in query.iter_mut() {
        actions.retain_mut(|mut delayed_action| {
            if delayed_action.timer.tick(time.delta()).just_finished() {
                commands.append(&mut delayed_action.action);
            }
            !delayed_action.timer.finished()
        });
    }
}
