use bevy::{
    app::AppExit,
    ecs::{
        event::EventWriter,
        query::With,
        system::{Query, Res},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::Vec2,
};
use bevy_rapier2d::prelude::Velocity;

const MOVE_SPEED: f32 = 500.0;
const BOOST_MULTIPLIER: f32 = 2.5;

pub fn boost(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<crate::player::PlayerMarker>>,
) {
    if buttons.pressed(KeyCode::Space) {
        let velocity = &mut query.single_mut().expect("no player").linvel;
        let normalized = velocity.normalize_or_zero();
        *velocity += normalized * (MOVE_SPEED * (BOOST_MULTIPLIER - 1.));
    }
}

/// System handling player movement according to WASD keyboard input.
pub fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<crate::player::PlayerMarker>>,
) {
    let velocity = &mut query.single_mut().expect("no player").linvel;

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

    // only need to do math if there is some input
    if let Some(input) = input {
        // velocity from input
        let input_delta = input.normalize_or_zero() * MOVE_SPEED;

        // don't allow to move faster than move speed or current speed
        // (if we are currently moving faster due to some external cause)
        let speed_limit = f32::max(velocity.length(), MOVE_SPEED);

        // if we simply add the input velocity
        let velocity_with_input = *velocity + input_delta;

        // if we would violate the speed limit
        if velocity_with_input.length() > speed_limit {
            // cap at speed limit, but use new angle
            *velocity = velocity_with_input.normalize_or_zero() * speed_limit;
        } else {
            // use the simple addition
            *velocity = velocity_with_input;
        }
    }
}

/// System handling Ctrl+Q quit.
pub fn input_quit(buttons: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<AppExit>) {
    if buttons.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        event_writer.write(AppExit::Success);
    }
}
