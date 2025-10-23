use std::borrow::Cow;

use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

use crate::simulation::thingy::ThingyNet;

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

#[derive(Debug, thiserror::Error)]
#[error("Message would exceed max size of {max}B, currently {current}B")]
pub(crate) struct MessageWouldExceedMax {
    current: usize,
    max: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct NetMessage<T: Serialize> {
    pub(crate) header: NetHeader,
    body_elements: Vec<T>,
}

impl<T: Serialize> NetMessage<T> {
    const MAX_SIZE: usize = 508;

    pub(crate) fn new(header: NetHeader) -> Self {
        Self {
            header,
            body_elements: Default::default(),
        }
    }

    pub(crate) fn try_push(&mut self, message: T) -> Result<(), MessageWouldExceedMax> {
        let current_size = self.current_serialized_size();
        if current_size + serialized_size(&message) > Self::MAX_SIZE {
            return Err(MessageWouldExceedMax {
                current: current_size,
                max: Self::MAX_SIZE,
            });
        }
        self.body_elements.push(message);
        Ok(())
    }

    pub(crate) fn body_elements(self) -> Vec<T> {
        self.body_elements
    }

    fn current_serialized_size(&self) -> usize {
        let mut size = 0;
        size += serialized_size(&self.header);
        for elem in &self.body_elements {
            size += serialized_size(elem);
        }

        size
    }
}

pub(crate) type ServerMessage<'data> = NetMessage<ServerBodyElement<'data>>;
pub(crate) type ClientMessage<'data> = NetMessage<ClientBodyElement<'data>>;

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

fn serialized_size(ser: &impl Serialize) -> usize {
    postcard::serialize_with_flavor(ser, postcard::ser_flavors::Size::default()).unwrap()
}
