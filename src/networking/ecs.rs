use bevy::{
    ecs::{bundle::Bundle, component::Component, query::QueryData},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;
use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::simulation::{player::PlayerNet, thingy::ThingyNet};

use super::tick::GameTick;

/*
TODO:
whole-state updates of entities
- atomic (if an update is sent for an entity, it contains entire state of entity)
  - no delta compression? so server doesn't need to keep track of client's last state
  - this requires querying several components (horizontal query!)
    - maybe query each component separately, then merge into coherent packet
  - client-side, store networked entity data in own component? or separate
    - probably not a big deal?
    - whichever is more ergonomic?
- (later) for each client, send different entities based on priority
    - priority based on
        - distance from "player"
        - client ACK'd entity update recently or not
        - game importance


client system order:
- fetch messages from network
- query entities based on net id
    - if no entity exists: "spawn" new entity with real + networked components
    - if entity exists: push networked components onto it
- for each component+networked component pair
    - query entities, update/interpolate real component based on networked component


*/

#[derive(Debug, Deserialize, Serialize, Component, Clone, Copy, Default, From)]
#[serde(transparent)]
pub(crate) struct Networked<T>(#[from] pub(crate) T);

#[derive(Debug, Component, Clone, Copy)]
pub(crate) struct LastNetUpdate(pub(crate) GameTick);

#[derive(
    Debug, Deserialize, Serialize, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub(crate) struct NetId(u128);

impl NetId {
    // TODO: track net-ids to avoid collisions?? 128 bit random should be fine tho
    pub(crate) fn new_random() -> Self {
        Self(rand::random())
    }
}

/// Wrapper enum for all possible net objects.
#[derive(Debug, Deserialize, Serialize, From)]
pub(crate) enum NetObj {
    Player(#[from] PlayerNet),
    Thingy(#[from] ThingyNet),
}

#[derive(Debug, Default, Bundle, Deserialize, Serialize)]
pub(crate) struct NetPhysicsBundle {
    pub(crate) transform: Networked<Transform>,
    pub(crate) velocity: Networked<Velocity>,
}

#[derive(Debug, QueryData)]
#[query_data(derive(Debug))]
pub(crate) struct NetPhysicsBundleQuery {
    pub(crate) transform: &'static Transform,
    pub(crate) velocity: &'static Velocity,
}

impl<'a> From<NetPhysicsBundleQueryItem<'a>> for NetPhysicsBundle {
    fn from(query: NetPhysicsBundleQueryItem) -> Self {
        Self {
            transform: (*query.transform).into(),
            velocity: (*query.velocity).into(),
        }
    }
}
