use bevy::{
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        query::{With, Without},
        system::{Commands, Query},
    },
    transform::components::Transform,
};

use crate::player::PlayerMarker;

/// Marker for game camera.
#[derive(Component)]
pub struct GameCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

pub fn camera_follow_player(
    mut camera: Query<&mut Transform, (With<GameCamera>, Without<PlayerMarker>)>,
    player: Query<&Transform, (With<PlayerMarker>, Without<GameCamera>)>,
) {
    let player_transform = player.single().expect("no player found");

    let mut camera_transform = camera.single_mut().expect("no camera found");

    camera_transform.translation = player_transform.translation;
}
