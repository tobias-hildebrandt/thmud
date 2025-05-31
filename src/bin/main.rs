use bevy::{
    DefaultPlugins,
    app::{App, FixedUpdate, Startup},
};
use thmud::{input_quit, movement, startup_spawn};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup_spawn)
        .add_systems(FixedUpdate, (movement, input_quit))
        .run();
}
