use bevy::app::AppExit;

use bevy::ecs::event::EventWriter;

use bevy::log::tracing;
use bevy::math::Vec2;

use bevy::ecs::query::With;

use bevy_rapier2d::prelude::Velocity;

use bevy::ecs::system::Query;

use bevy::input::keyboard::KeyCode;

use bevy::input::ButtonInput;

use bevy::ecs::system::Res;

/// System handling player movement according to WASD keyboard input.
pub fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<crate::player::PlayerMarker>>,
) {
    let mut velocity = query.single_mut().expect("no player");

    let mut input = Vec2::ZERO;

    const MOVE_SPEED: f32 = 500.0;

    if buttons.pressed(KeyCode::KeyW) {
        input.y += 1.0;
    }
    if buttons.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    }

    input = input.normalize_or_zero();

    if input.length() > 0.1 {
        let input_delta = input * MOVE_SPEED;

        let new = velocity.linvel + input_delta;

        let doesnt_surpass = (new.length() - 0.1) <= MOVE_SPEED;

        let slows_down = (new.length() - 0.1) < velocity.linvel.length();

        // tracing::info!(
        //     "simple?: {}, old: {:?} len {}, input: {:?} len {}, new: {:?} len {}",
        //     if simple { "y" } else { "n" },
        //     velocity.linvel,
        //     velocity.linvel.length(),
        //     input_delta,
        //     input_delta.length(),
        //     new,
        //     new.length(),
        // );

        // use simple addition only if the result is not greater than max move speed OR it slows down the player
        if doesnt_surpass || slows_down {
            velocity.linvel = new;
        } else if velocity.linvel.length() < MOVE_SPEED {
            velocity.linvel = new.normalize_or_zero() * MOVE_SPEED;
        }
        // else use old magnitude with new angle
        else {
            velocity.linvel = Vec2::from_angle(new.to_angle()) * velocity.linvel.length();
        }
    }
}

/// System handling Ctrl+Q quit.
pub fn input_quit(buttons: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<AppExit>) {
    if buttons.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        event_writer.write(AppExit::Success);
    }
}
