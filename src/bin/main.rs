use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, PluginGroup, Startup},
    math::Vec2,
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierContextInitialization, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use iyes_perf_ui::{PerfUiPlugin, prelude::PerfUiAllEntries};
use thmud::{
    input::{boost, input_quit, movement},
    physics::friction,
    startup::startup_spawn,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "some bevy game".into(),
                name: Some("some bevy game".into()),
                resolution: (1920., 900.).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }))
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
        .add_plugins((
            PerfUiPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
        ))
        .add_systems(Startup, |mut commands: bevy::ecs::system::Commands| {
            commands.spawn(PerfUiAllEntries::default());
        })
        .add_systems(FixedUpdate, (boost, movement, friction, input_quit))
        .run();
}
