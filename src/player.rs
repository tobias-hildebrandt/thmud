use bevy::ecs::component::Component;
use bevy::ecs::hierarchy::Children;

use bevy::ecs::spawn::SpawnRelated;
use bevy::sprite::ColorMaterial;
use bevy_rapier2d::prelude::CharacterLength;

use bevy::ecs::bundle::Bundle;

use bevy::render::mesh::Mesh;

use bevy::asset::Handle;

use bevy::text::TextColor;

use bevy::text::Text2d;

use bevy_rapier2d::prelude::ColliderMassProperties;

use bevy::transform::components::Transform;

use bevy_rapier2d::prelude::KinematicCharacterController;

use bevy_rapier2d::prelude::Collider;

use bevy_rapier2d::prelude::LockedAxes;
use bevy_rapier2d::prelude::RigidBody;

use bevy::sprite::MeshMaterial2d;

use bevy::render::mesh::Mesh2d;
use bevy_rapier2d::prelude::Velocity;

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
