use std::f32::consts::TAU;

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
use bevy_rapier2d::prelude::{CharacterLength, Collider, KinematicCharacterController, RigidBody};

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

    // spawn player
    let player = commands
        .spawn((
            Player,
            Mesh2d(meshes.add(Circle::new(50.0))),
            Collider::ball(50.0),
            RigidBody::Dynamic,
            KinematicCharacterController {
                offset: CharacterLength::Absolute(0.1),
                ..Default::default()
            },
            MeshMaterial2d(materials.add(Color::hsl(180., 0.95, 0.7))),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    // spawn player text
    let text = commands
        .spawn((
            Text2d("Player".to_string()),
            TextColor::BLACK,
            Transform::from_xyz(0.0, 20.0, 0.0),
        ))
        .id();

    // make text a child of player
    commands.entity(player).add_child(text);

    // create walls
    const NUM_WALLS: u32 = 7;
    const WALL_RADIUS: f32 = 300.;
    for i in 0..NUM_WALLS {
        let angle = (TAU / (NUM_WALLS as f32)) * i as f32;
        let x = angle.cos() * WALL_RADIUS;
        let y = angle.sin() * WALL_RADIUS;

        let mut wall = commands.spawn((Transform::from_xyz(x, y, 0.0), RigidBody::Fixed));

        match i % 2 {
            0 => wall.insert(Collider::ball(50.)),
            1 => wall.insert(Collider::cuboid(25.0, 25.0)),
            _ => unreachable!(),
        };
    }
}

/// System handling player movement according to WASD keyboard input.
pub fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut KinematicCharacterController, With<Player>>,
) {
    let mut controller = query.single_mut().expect("no player");

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

    controller.translation = Some(vel);
}

/// System handling Ctrl+Q quit.
pub fn input_quit(buttons: Res<ButtonInput<KeyCode>>, mut event_writer: EventWriter<AppExit>) {
    if buttons.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        event_writer.write(AppExit::Success);
    }
}
