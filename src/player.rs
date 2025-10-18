use bevy::{
    asset::Handle,
    ecs::{
        bundle::Bundle,
        component::Component,
        hierarchy::Children,
        query::AnyOf,
        spawn::SpawnRelated,
        system::{Commands, Query, Res},
    },
    render::mesh::{Mesh, Mesh2d},
    sprite::{ColorMaterial, MeshMaterial2d},
    text::{Text2d, TextColor},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{
    Ccd, Collider, ColliderMassProperties, ExternalForce, LockedAxes, RigidBody, Velocity,
};

use crate::assets::AssetHandles;

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
    pub(crate) external_force: ExternalForce,
    pub(crate) collision_detection: Ccd,

    // internal input forces
    pub(crate) input_force: MovementInputForce,
    pub(crate) boost_force: BoostForce,
}

#[derive(Debug, Default, Component)]
pub struct MovementInputForce(pub ExternalForce);

#[derive(Debug, Default, Component)]
pub struct BoostForce(pub ExternalForce);

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
            external_force: Default::default(),
            collision_detection: Default::default(),

            input_force: Default::default(),
            boost_force: Default::default(),
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

pub fn apply_player_forces(
    query: Query<(
        &mut ExternalForce,
        AnyOf<(&MovementInputForce, &BoostForce)>,
    )>,
) {
    for (mut total, (input, boost)) in query {
        *total = input.map(|w| w.0).unwrap_or_default() + boost.map(|w| w.0).unwrap_or_default();
    }
}

pub fn spawn_player(mut commands: Commands, asset_handles: Res<AssetHandles>) {
    // spawn player bundle
    commands.spawn(crate::player::Player::create_bundle(
        asset_handles.player_mesh.clone(),
        asset_handles.player_mat.clone(),
    ));
}
