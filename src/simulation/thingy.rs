use std::collections::BTreeSet;

use bevy::{
    app::{FixedUpdate, Plugin, Startup},
    ecs::{
        bundle::Bundle,
        component::Component,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, RigidBody, Velocity};
use serde::{Deserialize, Serialize};

use crate::{
    networking::ecs::{NetId, NetPhysicsObjectBundle},
    simulation::player::Player,
};

#[derive(Debug, Component)]
pub(crate) struct ThingyMarker;

#[derive(Debug, Bundle)]
pub(crate) struct Thingy {
    pub(crate) marker: ThingyMarker,
    pub(crate) transform: Transform,
    pub(crate) velocity: Velocity,
    pub(crate) rigid_body: RigidBody,
    pub(crate) collider: Collider,
    pub(crate) mass_properties: ColliderMassProperties,
    pub(crate) net: ThingyNet,
}

// network-synchronized state of thingy
// must contain all (dynamic, non-static) information the client needs to instantiate
// i.e. anything that could ever change and cannot be inferred based on other information
#[derive(Debug, Bundle, Serialize, Deserialize)]
pub(crate) struct ThingyNet {
    pub(crate) net_id: NetId,
    // TODO: technically unnecessary on server
    pub(crate) physics: NetPhysicsObjectBundle,
}

impl Thingy {
    const RADIUS: f32 = 25.0;
    const DENSITY: f32 = Player::DENSITY / 2.0;

    pub(crate) fn networked_bundle(net: ThingyNet) -> Self {
        Self {
            marker: ThingyMarker,
            transform: net.physics.transform.0,
            velocity: net.physics.velocity.0,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(Self::RADIUS, Self::RADIUS),
            mass_properties: ColliderMassProperties::Density(Self::DENSITY),
            net,
        }
    }
}
