use std::borrow::Cow;

use bevy::{
    ecs::{component::Component, resource::Resource},
    transform::components::Transform,
};
use bevy_rapier2d::prelude::Velocity;
use serde::{Deserialize, Serialize};

use crate::simulation::thingy::ThingyNet;

use super::ecs::{NetId, NetPhysicsObjectBundle};

#[derive(Debug, Resource)]
pub(crate) struct MessageBuffer<T> {
    pub(crate) messages: Vec<T>,
}

impl<T> Default for MessageBuffer<T> {
    fn default() -> Self {
        Self {
            messages: Default::default(),
        }
    }
}

impl<T> MessageBuffer<T> {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientMessage<'data> {
    pub(crate) header: NetHeader,
    pub(crate) body: Vec<ClientBodyElement<'data>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerMessage<'data> {
    pub(crate) header: NetHeader,
    pub(crate) body: Vec<ServerBodyElement<'data>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NetHeader {}

// TODO: maybe the elements will never have referenced data?

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ClientBodyElement<'data> {
    Dummy(Cow<'data, ()>),
    Register,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerBodyElement<'data> {
    Dummy(Cow<'data, ()>),
    Thingy(ThingyNet),
}
