use bevy::{
    ecs::{bundle::Bundle, component::Component},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize, Component, Clone, Copy, Default)]
pub(crate) struct Networked<T: Component>(pub(crate) T);

impl<T: Component> From<T> for Networked<T> {
    fn from(component: T) -> Self {
        Self(component)
    }
}

#[derive(Debug, Deserialize, Serialize, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NetId(pub(crate) u128);

impl From<u128> for NetId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<NetId> for u128 {
    fn from(value: NetId) -> Self {
        value.0
    }
}

#[derive(Debug, Bundle, Deserialize, Serialize)]
pub(crate) struct NetPhysicsObjectBundle {
    pub(crate) transform: Networked<Transform>,
    pub(crate) velocity: Networked<Velocity>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub enum MassProperties {
//     Density(f32),
//     Mass(f32),
// }

// impl From<MassProperties> for ColliderMassProperties {
//     fn from(value: MassProperties) -> Self {
//         match value {
//             MassProperties::Density(d) => Self::Density(d),
//             MassProperties::Mass(m) => Self::Mass(m),
//         }
//     }
// }
