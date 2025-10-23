use bevy::{
    DefaultPlugins, MinimalPlugins,
    app::{App, PluginGroup},
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};
use thmud::{
    assets::GameAssetPlugin,
    camera::GameCameraPlugin,
    debug::GameDebugPlugin,
    input::GameInputPlugin,
    networking::{client::ClientPlugin, server::ServerPlugin},
    simulation::{
        physics::GamePhysicsPlugin, player::GamePlayerPlugin, world_gen::GameWorldGenPlugin,
    },
};

fn main() {
    let mut app = App::new();

    // game plugins
    app.add_plugins((
        GameAssetPlugin,
        GameCameraPlugin,
        GameInputPlugin,
        GamePhysicsPlugin,
        GamePlayerPlugin,
        GameDebugPlugin,
    ));

    let args: String = std::env::args().skip(1).collect();
    let name = format!("thmud {args}");

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

    if std::env::args().any(|a| a == "client") {
        app.add_plugins(ClientPlugin);
    } else {
        // TODO: remove unnecessary plugins
        // app.add_plugins(DefaultPlugins);
        app.add_plugins(ServerPlugin);
        app.add_plugins(GameWorldGenPlugin);
    }

    app.run();
}
