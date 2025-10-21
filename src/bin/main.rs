use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup},
    window::{MonitorSelection, Window, WindowPlugin, WindowPosition},
};
use thmud::{
    assets::GameAssetPlugin, camera::GameCameraPlugin, debug::GameDebugPlugin,
    input::GameInputPlugin, physics::GamePhysicsPlugin, player::GamePlayerPlugin,
    thingy::GameThingyPlugin,
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
        // game plugins
        .add_plugins((
            GameAssetPlugin,
            GameCameraPlugin,
            GameInputPlugin,
            GamePhysicsPlugin,
            GamePlayerPlugin,
            GameThingyPlugin,
            GameDebugPlugin,
        ))
        .run();
}
