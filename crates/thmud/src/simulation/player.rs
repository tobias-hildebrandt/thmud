use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        bundle::Bundle,
        component::Component,
        hierarchy::Children,
        query::{QueryData, With},
        spawn::SpawnRelated,
        system::Query,
    },
    render::view::Visibility,
    text::{Text2d, TextColor},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{
    Ccd, Collider, ColliderMassProperties, ExternalForce, LockedAxes, RigidBody, Velocity,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::networking::serde_helpers::{ExternalForceSerde, FromIntoNetworked};
use crate::networking::{
    ecs::{LastNetUpdate, NetId, NetPhysicsBundle, NetPhysicsBundleQuery, Networked},
    tick::GameTick,
};

use super::input::PlayerInput;

/// Marker struct for players.
#[derive(Debug, Component)]
pub(crate) struct PlayerMarker;

/// Marker struct for local player, only used by client.
#[derive(Debug, Component)]
pub(crate) struct LocalPlayerMarker;

// TODO: unnecessary? net id should work fine in basically every case
#[derive(Debug, Deserialize, Serialize, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PlayerId(u128);

impl PlayerId {
    pub(crate) fn new_random() -> Self {
        Self(rand::random())
    }
}

// TODO: move graphics out
#[derive(Debug, Bundle)]
pub(crate) struct Player {
    // game
    pub(crate) player: PlayerMarker,

    // physics
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) velocity: Velocity,
    pub(crate) transform: Transform,
    pub(crate) mass_properties: ColliderMassProperties,
    pub(crate) locked_axes: LockedAxes,
    pub(crate) total_external_force: ExternalForce,
    pub(crate) collision_detection: Ccd,

    // input
    pub(crate) input: PlayerInput,
    pub(crate) input_force: MovementInputForce, // sub-force of total_external_force
}

#[serde_as]
#[derive(Debug, Bundle, Serialize, Deserialize)]
pub(crate) struct PlayerNet {
    pub(crate) net_id: NetId,
    pub(crate) physics: NetPhysicsBundle,
    pub(crate) player_id: Networked<PlayerId>,
    #[serde_as(as = "FromIntoNetworked<ExternalForceSerde>")]
    pub(crate) total_external_force: Networked<ExternalForce>,
    pub(crate) input: Networked<PlayerInput>,
}

impl PlayerNet {
    pub(crate) fn new_random() -> Self {
        Self {
            net_id: NetId::new_random(),
            physics: Default::default(),
            player_id: PlayerId::new_random().into(),
            total_external_force: Default::default(),
            input: Default::default(),
        }
    }
}

#[derive(Debug, QueryData)]
#[query_data(derive(Debug, Clone))]
pub(crate) struct PlayerNetQuery {
    pub(crate) marker: &'static PlayerMarker,
    pub(crate) net_id: &'static NetId,
    pub(crate) physics: NetPhysicsBundleQuery,
    pub(crate) player_id: &'static Networked<PlayerId>,
    pub(crate) total_external_force: &'static ExternalForce,
    pub(crate) input: &'static PlayerInput,
}

impl<'a> From<PlayerNetQueryItem<'a>> for PlayerNet {
    fn from(query: PlayerNetQueryItem) -> Self {
        Self {
            net_id: *query.net_id,
            physics: query.physics.into(),
            player_id: *query.player_id,
            total_external_force: (*query.total_external_force).into(),
            input: (*query.input).into(),
        }
    }
}

#[derive(Debug, Default, Component)]
pub(crate) struct MovementInputForce(pub(crate) ExternalForce);

#[derive(Debug, Bundle)]
pub(crate) struct PlayerText {
    pub(crate) text: Text2d,
    pub(crate) color: TextColor,
    pub(crate) transform: Transform,
}

impl Player {
    pub(crate) const DENSITY: f32 = 20.;

    pub(crate) fn client_bundle(net: PlayerNet, last_updated: GameTick) -> impl Bundle {
        // player entity
        let player = Player {
            player: PlayerMarker,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(50.0),
            velocity: net.physics.velocity.0,
            transform: net.physics.transform.0,
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            total_external_force: Default::default(),
            collision_detection: Default::default(),

            input_force: Default::default(),
            input: net.input.0,
        };

        // child entity for text
        let text = PlayerText {
            text: Text2d(hex::encode_upper(net.player_id.0.0.to_le_bytes())),
            color: TextColor::BLACK,
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
        };

        // TODO: re-add graphics

        // client-specific components to add to player entity
        let client_specific = (net, LastNetUpdate(last_updated));

        (
            player,
            client_specific,
            Visibility::Visible, // otherwise PlayerText complains
            Children::spawn_one(text),
        )
    }

    pub(crate) fn server_bundle(net: PlayerNet) -> impl Bundle {
        // player entity
        let player = Player {
            player: PlayerMarker,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::ball(50.0),
            velocity: net.physics.velocity.0,
            transform: net.physics.transform.0,
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            total_external_force: Default::default(),
            collision_detection: Default::default(),

            input_force: Default::default(),
            input: net.input.0,
        };

        (player, net.net_id, net.player_id)
    }
}

fn sum_subforces(query: Query<(&mut ExternalForce, &MovementInputForce), With<PlayerMarker>>) {
    for (mut total, input) in query {
        // TODO: check if client,
        *total = input.0;
    }
}

pub(crate) struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, sum_subforces);
    }
}
