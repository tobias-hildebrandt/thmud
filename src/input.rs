use bevy::{
    app::{AppExit, FixedUpdate, Plugin},
    ecs::{
        event::EventWriter,
        query::With,
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::Vec2,
};
use bevy_rapier2d::prelude::{ExternalForce, Velocity};

use crate::player::{BoostForce, MovementInputForce};

const MILLION: f32 = 1_000_000.;
const MOVE_FORCE: f32 = 500. * MILLION;
const BOOST_FORCE: f32 = 1_000. * MILLION;

/// System handling player boost according to current velocity direction.
fn boost(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut BoostForce, &Velocity), With<crate::player::PlayerMarker>>,
) {
    let (mut force, velocity) = query.single_mut().expect("no player");
    if buttons.pressed(KeyCode::Space) {
        let velocity_normalized = velocity.linvel.normalize_or_zero();

        force.0 = ExternalForce {
            force: velocity_normalized * BOOST_FORCE,
            torque: Default::default(),
        };
    } else {
        force.0 = Default::default();
    }
}

/// System handling player movement according to WASD keyboard input.
fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut MovementInputForce, With<crate::player::PlayerMarker>>,
) {
    let mut force = query.single_mut().expect("no player");

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

    /*
    NOTE:
    probably don't need to scale down based on current velocity, since at high velocities,
    friction/drag should apply a greater force than our input force
    */

    if let Some(input) = input {
        let f = input * MOVE_FORCE;
        force.0.force = f;
    } else {
        force.0.force = Default::default();
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
        app.add_systems(FixedUpdate, (movement, boost.after(movement), input_quit));
    }
}
