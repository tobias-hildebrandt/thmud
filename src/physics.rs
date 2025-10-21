use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::system::Query,
    math::Vec2,
};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierContextInitialization, RapierPhysicsPlugin},
    prelude::Velocity,
};

const FRICTION_COEFFICIENT: f32 = 0.9;
const STOP_SPEED: f32 = 0.5;

fn friction(mut query: Query<&mut Velocity>) {
    for mut vel in &mut query {
        vel.linvel *= FRICTION_COEFFICIENT;
        vel.angvel *= FRICTION_COEFFICIENT;

        // make sure objects come to a stop
        if vel.linvel.length() < STOP_SPEED {
            vel.linvel = Vec2::ZERO;
        }
    }
}

pub struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            // rapier physics
            .add_plugins({
                let mut config = RapierConfiguration::new(1.);
                config.gravity = Vec2::ZERO;
                RapierPhysicsPlugin::<NoUserData>::default().with_custom_initialization(
                    RapierContextInitialization::InitializeDefaultRapierContext {
                        integration_parameters: Default::default(),
                        rapier_configuration: config,
                    },
                )
            })
            // custom game physics
            .add_systems(FixedUpdate, friction);
    }
}
