use bevy::ecs::system::Query;
use bevy_rapier2d::prelude::Velocity;

pub fn friction(mut query: Query<&mut Velocity>) {
    for mut vel in &mut query {
        vel.linvel *= 0.9;
        vel.angvel *= 0.9;
    }
}
