use std::{collections::HashMap, f32::consts::TAU};

use bevy::{
    app::AppExit,
    asset::{Assets, Handle},
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventWriter,
        hierarchy::Children,
        query::With,
        resource::Resource,
        spawn::SpawnRelated,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::{
        Quat, Vec2, Vec3,
        primitives::{Circle, Rectangle},
    },
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    text::{Text2d, TextColor},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{
    CharacterLength, Collider, ColliderMassProperties, KinematicCharacterController, RigidBody,
};
use fixed::{FixedI32, types::extra::U12};

/// Marker struct for players.
#[derive(Debug, Component)]
pub struct PlayerMarker;

#[derive(Debug, Bundle)]
pub struct Player {
    // game
    player: PlayerMarker,

    // graphics
    mesh: Mesh2d,
    mesh_material: MeshMaterial2d<ColorMaterial>,

    // physics
    rigid_body: RigidBody,
    collider: Collider,
    kcc: KinematicCharacterController,
    transform: Transform,
    mass_properties: ColliderMassProperties,
}

#[derive(Debug, Bundle)]
pub struct PlayerText {
    text: Text2d,
    color: TextColor,
    transform: Transform,
}

impl Player {
    const DENSITY: f32 = 20.;

    pub(crate) fn create_bundle(
        mesh: Handle<Mesh>,
        material: Handle<ColorMaterial>,
    ) -> impl Bundle {
        // player entity
        let player = Player {
            player: PlayerMarker,
            mesh: Mesh2d(mesh),
            mesh_material: MeshMaterial2d(material),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(50.0),
            kcc: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.1),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
        };

        // child entity for text
        let text = PlayerText {
            text: Text2d("Player".to_string()),
            color: TextColor::BLACK,
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
        };

        (player, Children::spawn_one(text))
    }
}

#[derive(Debug, Component)]
pub struct ThingyMarker;

#[derive(Debug, Bundle)]
pub struct Thingy {
    marker: ThingyMarker,
    mesh: Mesh2d,
    mesh_material: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    rigid_body: RigidBody,
    collider: Collider,
    mass_properties: ColliderMassProperties,
}

impl Thingy {
    pub(crate) fn create_bundle_polar(
        fixed: bool,
        angle: f32,
        distance_from_origin: f32,
        is_circle: bool,
        color: Handle<ColorMaterial>,
        mesh: Handle<Mesh>,
    ) -> Self {
        let x = angle.cos() * distance_from_origin;
        let y = angle.sin() * distance_from_origin;

        Self {
            marker: ThingyMarker,
            transform: Transform::from_xyz(x, y, 0.0),
            mesh: Mesh2d(mesh),
            mesh_material: MeshMaterial2d(color),
            rigid_body: if fixed {
                RigidBody::Fixed
            } else {
                RigidBody::Dynamic
            },
            collider: match is_circle {
                true => Collider::ball(50.),
                false => Collider::cuboid(25.0, 25.0),
            },
            mass_properties: ColliderMassProperties::Density(Player::DENSITY / 2.),
        }
    }
}

#[derive(Resource)]
struct AssetHandles {
    player_mesh: Handle<Mesh>,
    player_mat: Handle<ColorMaterial>,
    thingy_circle_meshes: HashMap<FixedI32<U12>, Handle<Mesh>>,
    thingy_square_meshes: HashMap<FixedI32<U12>, Handle<Mesh>>,
    thingy_mat: Handle<ColorMaterial>,
}

impl AssetHandles {
    fn thingy_circle_mesh(&mut self, radius: f32, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.thingy_circle_meshes
            .entry(FixedI32::from_num(radius))
            .or_insert(meshes.add(Circle::new(radius)))
            .clone()
    }

    fn thingy_square_mesh(&mut self, side: f32, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.thingy_square_meshes
            .entry(FixedI32::from_num(side))
            .or_insert(meshes.add(Rectangle::new(side, side)))
            .clone()
    }
}

/// Spawn starting entities.
pub fn startup_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // create assets
    let mut asset_handles = AssetHandles {
        player_mesh: meshes.add(Circle::new(50.0)),
        player_mat: materials.add(Color::hsl(180., 0.95, 0.7)),
        thingy_circle_meshes: HashMap::new(),
        thingy_square_meshes: HashMap::new(),
        thingy_mat: materials.add(Color::hsl(110., 0.45, 0.4)),
    };

    // spawn player bundle
    commands.spawn(Player::create_bundle(
        asset_handles.player_mesh.clone(),
        asset_handles.player_mat.clone(),
    ));

    // create inner walls
    const NUM_WALLS: u32 = 7;
    const WALL_DISTANCE_FROM_CENTER: f32 = 300.;
    for i in 0..NUM_WALLS {
        let angle = (TAU / (NUM_WALLS as f32)) * i as f32;
        commands.spawn(Thingy::create_bundle_polar(
            true,
            angle,
            WALL_DISTANCE_FROM_CENTER,
            i % 2 == 0,
            asset_handles.thingy_mat.clone(),
            asset_handles.thingy_square_mesh(20., &mut meshes),
        ));
    }

    // create outer walls
    for (index, (x, y)) in [(0, 400), (400, 0), (0, -400), (-400, 0)]
        .iter()
        .enumerate()
    {
        // TODO: move to function
        commands.spawn(Thingy {
            marker: ThingyMarker,
            transform: Transform {
                translation: Vec3::new(*x as f32, *y as f32, 0.),
                rotation: Quat::from_rotation_z(if index % 2 == 0 { 0. } else { TAU / 4. }),
                ..Default::default()
            },
            rigid_body: RigidBody::Fixed,
            collider: Collider::cuboid(400., 20.),
            mass_properties: ColliderMassProperties::Density(Player::DENSITY / 2.),
            mesh: Mesh2d(asset_handles.thingy_circle_mesh(20., &mut meshes)),
            mesh_material: MeshMaterial2d(asset_handles.thingy_mat.clone()),
        });
    }

    // create balls
    const NUM_BALLS: u32 = 9;
    const BALL_DISTANCE_FROM_CENTER: f32 = 200.;
    for i in 0..NUM_BALLS {
        let angle = (TAU / (NUM_BALLS as f32)) * i as f32;
        commands.spawn(Thingy::create_bundle_polar(
            false,
            angle,
            BALL_DISTANCE_FROM_CENTER,
            i % 2 == 0,
            asset_handles.thingy_mat.clone(),
            asset_handles.thingy_circle_mesh(15., &mut meshes),
        ));
    }

    // insert handles for later use
    commands.insert_resource(asset_handles);
}

/// System handling player movement according to WASD keyboard input.
pub fn movement(
    buttons: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut KinematicCharacterController, With<PlayerMarker>>,
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
