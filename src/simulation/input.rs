use bevy::{
    app::{AppExit, FixedPreUpdate, FixedUpdate, Plugin},
    ecs::{
        component::Component,
        event::EventWriter,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::Vec2,
};

use serde::{Deserialize, Serialize};

use crate::simulation::player::{MovementInputForce, PlayerMarker};

use super::player::LocalPlayerMarker;

const MILLION: f32 = 1_000_000.;
const MOVE_FORCE: f32 = 500. * MILLION;

#[derive(Debug, Component, Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct PlayerInput {
    movement: Option<Vec2>,
}

/// Add local inputs to local player entity.
pub(crate) fn read_local_inputs(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut PlayerInput, With<LocalPlayerMarker>>,
) {
    let Ok(mut local_player_input) = query.single_mut() else {
        return;
    };

    let mut input: Option<Vec2> = None;

    if buttons.pressed(KeyCode::KeyW) {
        input.get_or_insert_default().y += 1.0;
    }
    if buttons.pressed(KeyCode::KeyS) {
        input.get_or_insert_default().y -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyA) {
        input.get_or_insert_default().x -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyD) {
        input.get_or_insert_default().x += 1.0;
    };

    local_player_input.movement = input;
}

/// System handling player movement according to input.
fn apply_movement_inputs(
    query: Query<(&mut MovementInputForce, &PlayerInput), With<PlayerMarker>>,
) {
    /*
    NOTE:
    probably don't need to scale down based on current velocity, since at high velocities,
    friction/drag should apply a greater force than our input force
    */

    for (mut force, input) in query {
        if let Some(input) = input.movement {
            let f = input * MOVE_FORCE;
            force.0.force = f;
        } else {
            force.0.force = Default::default();
        }
    }
}

/// System handling Ctrl+Q quit.
fn input_quit(buttons: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<AppExit>) {
    if buttons.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        event_writer.write(AppExit::Success);
    }
}

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, apply_movement_inputs);
    }
}

pub struct GameLocalInputPlugin;

impl Plugin for GameLocalInputPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedPreUpdate,
            (read_local_inputs.before(apply_movement_inputs), input_quit),
        );
    }
}
