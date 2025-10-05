use bevy::math::Quat;
use bevy::math::primitives::Circle;
use bevy::sprite::ColorMaterial;
use bevy::sprite::MeshMaterial2d;

use bevy::render::mesh::Mesh2d;

use bevy_rapier2d::prelude::Collider;
use bevy_rapier2d::prelude::ColliderMassProperties;

use bevy_rapier2d::prelude::RigidBody;

use bevy::math::Vec3;

use bevy::transform::components::Transform;

use std::f32::consts::TAU;

use std::collections::HashMap;

use bevy::color::Color;

use bevy::core_pipeline::core_2d::Camera2d;

use bevy::render::mesh::Mesh;

use bevy::asset::Assets;

use bevy::ecs::system::ResMut;

use bevy::ecs::system::Commands;

/// Spawn starting entities.
pub fn startup_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // create assets
    let mut asset_handles = crate::assets::AssetHandles {
        player_mesh: meshes.add(Circle::new(50.0)),
        player_mat: materials.add(Color::hsl(180., 0.95, 0.7)),
        thingy_circle_meshes: HashMap::new(),
        thingy_square_meshes: HashMap::new(),
        thingy_mat: materials.add(Color::hsl(110., 0.45, 0.4)),
    };

    // spawn player bundle
    commands.spawn(crate::player::Player::create_bundle(
        asset_handles.player_mesh.clone(),
        asset_handles.player_mat.clone(),
    ));

    // create inner walls
    const NUM_WALLS: u32 = 7;
    const WALL_DISTANCE_FROM_CENTER: f32 = 300.;
    for i in 0..NUM_WALLS {
        let angle = (TAU / (NUM_WALLS as f32)) * i as f32;
        commands.spawn(crate::thingy::Thingy::create_bundle_polar(
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
        commands.spawn(crate::thingy::Thingy {
            marker: crate::thingy::ThingyMarker,
            transform: Transform {
                translation: Vec3::new(*x as f32, *y as f32, 0.),
                rotation: Quat::from_rotation_z(if index % 2 == 0 { 0. } else { TAU / 4. }),
                ..Default::default()
            },
            rigid_body: RigidBody::Fixed,
            collider: Collider::cuboid(400., 20.),
            mass_properties: ColliderMassProperties::Density(crate::player::Player::DENSITY / 2.),
            mesh: Mesh2d(asset_handles.thingy_circle_mesh(20., &mut meshes)),
            mesh_material: MeshMaterial2d(asset_handles.thingy_mat.clone()),
        });
    }

    // create balls
    const NUM_BALLS: u32 = 9;
    const BALL_DISTANCE_FROM_CENTER: f32 = 200.;
    for i in 0..NUM_BALLS {
        let angle = (TAU / (NUM_BALLS as f32)) * i as f32;
        commands.spawn(crate::thingy::Thingy::create_bundle_polar(
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
