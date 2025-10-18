use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, PluginGroup, Startup, Update},
    ecs::schedule::IntoScheduleConfigs,
    math::Vec2,
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierContextInitialization, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use iyes_perf_ui::{PerfUiPlugin, prelude::PerfUiDefaultEntries};
use thmud::{
    assets::initialize_assets,
    camera::{camera_follow_player, spawn_camera},
    input::{boost, input_quit, movement},
    physics::friction,
    player::{apply_player_forces, spawn_player},
    thingy::{chunk_spawning, initialize_chunk_spawn_tracker, initialize_world_seed},
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
        .add_systems(
            Startup,
            (
                spawn_camera,
                initialize_world_seed,
                initialize_assets,
                spawn_player.after(initialize_assets),
                initialize_chunk_spawn_tracker,
            ),
        )
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
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
        ))
        .add_systems(Startup, |mut commands: bevy::ecs::system::Commands| {
            commands.spawn(PerfUiDefaultEntries::default());
        })
        // game logic
        .add_systems(
            FixedUpdate,
            (
                boost,
                movement,
                friction,
                apply_player_forces,
                input_quit,
                chunk_spawning,
            ),
        )
        // render-only
        .add_systems(Update, camera_follow_player)
        .run();
}
