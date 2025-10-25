use bevy::{
    ecs::{bundle::Bundle, component::Component, query::QueryData},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, RigidBody, Velocity};
use serde::{Deserialize, Serialize};

use crate::{
    networking::ecs::{NetId, NetPhysicsBundle, NetPhysicsBundleQuery, NetQueryable},
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
    pub(crate) physics: NetPhysicsBundle,
}

#[derive(Debug, QueryData)]
#[query_data(derive(Debug))]
pub(crate) struct ThingyNetQuery {
    net_id: &'static NetId,
    physics: NetPhysicsBundleQuery,
}

impl<'a> ThingyNetQueryItem<'a> {
    pub(crate) fn transform(&self) -> Transform {
        *self.physics.transform
    }
}

impl<'a> NetQueryable<'a> for ThingyNet {
    type Query = ThingyNetQuery;
}

impl<'a> From<ThingyNetQueryItem<'a>> for ThingyNet {
    fn from(query: ThingyNetQueryItem) -> Self {
        Self {
            net_id: *query.net_id,
            physics: query.physics.into(),
        }
    }
}

impl Thingy {
    const RADIUS: f32 = 25.0;
    const DENSITY: f32 = Player::DENSITY / 2.0;

    pub(crate) fn bundle(net: ThingyNet) -> Self {
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
