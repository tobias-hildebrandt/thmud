use bevy::{
    DefaultPlugins,
    app::{
        App, PanicHandlerPlugin, PluginGroup, ScheduleRunnerPlugin, TaskPoolPlugin,
        TerminalCtrlCHandlerPlugin,
    },
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
    log::LogPlugin,
    state::app::StatesPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};
use thmud::{
    assets::GameAssetPlugin,
    camera::GameCameraPlugin,
    debug::GameDebugPlugin,
    input::GameInputPlugin,
    networking::{client::GameClientPlugin, server::GameServerPlugin},
    simulation::{physics::GamePhysicsPlugin, player::GamePlayerPlugin, world::GameWorldGenPlugin},
};

fn main() {
    let mut app = App::new();
    if std::env::args().any(|a| a == "client") {
        let name = "thmud client".to_string();

        // bevy plugins
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: name.clone(),
                name: Some(name.clone()),
                resolution: (1920., 900.).into(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }));

        // game plugins
        app.add_plugins(GameAssetPlugin);
        app.add_plugins(GameCameraPlugin);
        app.add_plugins(GameInputPlugin);
        app.add_plugins(GamePlayerPlugin);
        app.add_plugins(GameDebugPlugin);
        app.add_plugins(GamePhysicsPlugin);
        app.add_plugins(GameClientPlugin);
    } else {
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
            ScheduleRunnerPlugin::default(),
            TerminalCtrlCHandlerPlugin,
            StatesPlugin,
        ));

        // game plugins
        app.add_plugins(GamePhysicsPlugin);
        app.add_plugins(GameWorldGenPlugin);
        app.add_plugins(GameServerPlugin);
    }

    app.run();
}
