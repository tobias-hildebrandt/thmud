use bevy::{
    app::AppExit,
    asset::Assets,
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        component::Component,
        event::EventWriter,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::{Vec2, primitives::Circle},
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    text::{Text2d, TextColor},
    transform::components::Transform,
};

/// Marker struct for players.
#[derive(Debug, Component)]
pub struct Player;

/// Spawn starting entities.
pub fn startup_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let shape = meshes.add(Circle::new(50.0));
    let color = materials.add(Color::hsl(180., 0.95, 0.7));

    let player = commands
        .spawn((
            Player,
            Mesh2d(shape),
            MeshMaterial2d(color),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let text = commands
        .spawn((
            Text2d("Player".to_string()),
            TextColor::BLACK,
            Transform::from_xyz(0.0, 20.0, 0.0),
        ))
        .id();

    commands.entity(player).add_child(text);
}

/// System handling player movement according to WASD keyboard input.
pub fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut().expect("no player");

    let mut vel = Vec2::ZERO;

    const SPEED: f32 = 5.0;
    if buttons.pressed(KeyCode::KeyW) {
        vel.y += 1.0;
    }
    if buttons.pressed(KeyCode::KeyS) {
        vel.y -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyA) {
        vel.x -= 1.0;
    }
    if buttons.pressed(KeyCode::KeyD) {
        vel.x += 1.0;
    }
    vel = vel.normalize_or_zero() * SPEED;

    transform.translation.x += vel.x;
    transform.translation.y += vel.y;
}

/// System handling Ctrl+Q quit.
pub fn input_quit(buttons: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<AppExit>) {
    if buttons.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        event_writer.write(AppExit::Success);
    }
}
