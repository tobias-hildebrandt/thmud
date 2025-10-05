use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::render::mesh::Mesh;

use bevy::asset::Handle;

use bevy::sprite::ColorMaterial;
use bevy_rapier2d::prelude::ColliderMassProperties;

use bevy_rapier2d::prelude::Collider;

use bevy_rapier2d::prelude::RigidBody;

use bevy::transform::components::Transform;

use bevy::sprite::MeshMaterial2d;

use bevy::render::mesh::Mesh2d;

#[derive(Debug, Component)]
pub struct ThingyMarker;

#[derive(Debug, Bundle)]
pub struct Thingy {
    pub(crate) marker: ThingyMarker,
    pub(crate) mesh: Mesh2d,
    pub(crate) mesh_material: MeshMaterial2d<ColorMaterial>,
    pub(crate) transform: Transform,
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) mass_properties: ColliderMassProperties,
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
            mass_properties: ColliderMassProperties::Density(crate::player::Player::DENSITY / 2.),
        }
    }
}
