use bevy::{ecs::system::Query, math::Vec2};
use bevy_rapier2d::prelude::Velocity;

const FRICTION_COEFFICIENT: f32 = 0.9;
const STOP_SPEED: f32 = 0.5;

pub fn friction(mut query: Query<&mut Velocity>) {
    for mut vel in &mut query {
        vel.linvel *= FRICTION_COEFFICIENT;
        vel.angvel *= FRICTION_COEFFICIENT;

        // make sure objects come to a stop
        if vel.linvel.length() < STOP_SPEED {
            vel.linvel = Vec2::ZERO;
        }
    }
}
