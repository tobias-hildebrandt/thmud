use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, hierarchy::Children, spawn::SpawnRelated},
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    text::{Text2d, TextColor},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, LockedAxes, RigidBody, Velocity};

/// Marker struct for players.
#[derive(Debug, Component)]
pub struct PlayerMarker;

#[derive(Debug, Bundle)]
pub struct Player {
    // game
    pub(crate) player: PlayerMarker,

    // graphics
    pub(crate) mesh: Mesh2d,
    pub(crate) mesh_material: MeshMaterial2d<ColorMaterial>,

    // physics
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) velocity: Velocity,
    pub(crate) transform: Transform,
    pub(crate) mass_properties: ColliderMassProperties,
    pub(crate) locked_axes: LockedAxes,
}

#[derive(Debug, Bundle)]
pub struct PlayerText {
    pub(crate) text: Text2d,
    pub(crate) color: TextColor,
    pub(crate) transform: Transform,
}

impl Player {
    pub(crate) const DENSITY: f32 = 20.;

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
            velocity: Velocity::zero(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
            locked_axes: LockedAxes::ROTATION_LOCKED,
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
