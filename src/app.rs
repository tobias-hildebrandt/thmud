use crate::{
    display::{camera::GameCameraPlugin, debug::GameDebugUiPlugin, grid::GameDrawGridPlugin},
    networking::{
        client::GameClientPlugin, netrate::GameNetRatePlugin, server::GameServerPlugin,
        tick::GameTickPlugin,
    },
    simulation::{
        input::{GameInputPlugin, GameLocalInputPlugin},
        physics::GamePhysicsPlugin,
        player::GamePlayerPlugin,
        world::GameWorldGenPlugin,
    },
};
use bevy::{
    DefaultPlugins,
    app::{
        App, PanicHandlerPlugin, PluginGroup, ScheduleRunnerPlugin, TaskPoolPlugin,
        TerminalCtrlCHandlerPlugin,
    },
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin, LogDiagnosticsPlugin},
    ecs::resource::Resource,
    log::LogPlugin,
    state::app::StatesPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};

/// The run type of the process.
///
/// Not a bevy resource for ease of access. Must be [`Self::set`] at start of process.
#[derive(Debug, Clone, Copy, Resource)]
pub enum RunType {
    Client,
    Server,
}

/// Build and return the bevy [`App`] for the given [`RunType`].
pub fn app(run_type: RunType) -> App {
    let mut app = App::new();

    match run_type {
        RunType::Client => {
            let name = "thmud client".to_string();

            // bevy plugins
            app.add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: name.clone(),
                    name: Some(name.clone()),
                    resolution: (1280., 720.).into(),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..Default::default()
                }),
                ..Default::default()
            }));

            // set physics tick rate
            app.insert_resource(bevy::time::Time::<bevy::time::Fixed>::from_hz(60.0));

            // game plugins
            app.add_plugins(GameCameraPlugin);
            app.add_plugins(GameLocalInputPlugin);
            app.add_plugins(GameInputPlugin);
            app.add_plugins(GamePlayerPlugin);
            app.add_plugins(GameDebugUiPlugin);
            app.add_plugins(GamePhysicsPlugin);
            app.add_plugins(GameClientPlugin);
            app.add_plugins(GameNetRatePlugin::new(2));
            app.add_plugins(GameDrawGridPlugin);
        }
        RunType::Server => {
            // bevy plugins
            // TODO: re-assess necessary plugins
            app.add_plugins((
                PanicHandlerPlugin,
                LogPlugin::default(),
                TaskPoolPlugin::default(),
                FrameCountPlugin,
                TimePlugin,
                TransformPlugin,
                DiagnosticsPlugin,
                LogDiagnosticsPlugin::default(),
                ScheduleRunnerPlugin::default(),
                TerminalCtrlCHandlerPlugin,
                StatesPlugin,
            ));
            // set physics tick rate
            app.insert_resource(bevy::time::Time::<bevy::time::Fixed>::from_hz(60.0));

            // game plugins
            app.add_plugins(GamePhysicsPlugin);
            app.add_plugins(GameWorldGenPlugin);
            app.add_plugins(GameInputPlugin);
            app.add_plugins(GamePlayerPlugin);
            app.add_plugins(GameServerPlugin);
            app.add_plugins(GameTickPlugin);
            app.add_plugins(GameNetRatePlugin::new(2));
        }
    };
    app
}
