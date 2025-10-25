use bevy::{
    app::{Plugin, Startup, Update},
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        query::{With, Without},
        system::{Commands, Query},
    },
    transform::components::Transform,
};

use crate::simulation::player::LocalPlayerMarker;

/// Marker for game camera.
#[derive(Component)]
struct GameCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

// TODO: smooth
fn camera_follow_local_player(
    mut camera: Query<&mut Transform, With<GameCamera>>,
    local_player: Query<&Transform, (With<LocalPlayerMarker>, Without<GameCamera>)>,
) {
    let Ok(player_transform) = local_player.single() else {
        return;
    };

    let mut camera_transform = camera.single_mut().expect("no camera found");

    camera_transform.translation = player_transform.translation;
}

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, spawn_camera)
            // render tick
            .add_systems(Update, camera_follow_local_player);
    }
}
