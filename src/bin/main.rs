use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup},
    math::Vec2,
};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierContextInitialization, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use thmud::{input_quit, movement, startup_spawn};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup_spawn)
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
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(FixedUpdate, (movement, input_quit))
        .run();
}
