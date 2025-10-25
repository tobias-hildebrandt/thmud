use std::{borrow::Cow, collections::VecDeque};

use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

use crate::simulation::{
    input::PlayerInput,
    player::{PlayerId, PlayerNet},
    thingy::ThingyNet,
};

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
    body_elements: VecDeque<T>,
}

impl<T: Serialize> NetMessage<T> {
    // TODO: dynamically change this based on network behavior?
    const MAX_SIZE: usize = 508;

    pub(crate) fn new(header: NetHeader) -> Self {
        Self {
            header,
            body_elements: Default::default(),
        }
    }

    pub(crate) fn try_push_back(&mut self, message: T) -> Result<(), MessageWouldExceedMax> {
        let current_size = self.current_serialized_size();
        let message_size = serialized_size(&message);
        if current_size + message_size > Self::MAX_SIZE {
            return Err(MessageWouldExceedMax {
                current: current_size,
                max: Self::MAX_SIZE,
            });
        }
        self.body_elements.push_back(message);
        Ok(())
    }

    // evicts and returns tail element if message would exceed max
    // TODO: handle case where given message is too large all by itself
    pub(crate) fn push_front(&mut self, message: T) -> Option<T> {
        self.body_elements.push_front(message);
        if self.current_serialized_size() > Self::MAX_SIZE {
            self.body_elements.pop_back()
        } else {
            None
        }
    }

    pub(crate) fn body_elements(self) -> VecDeque<T> {
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
    Unregister,
    Input(PlayerInput),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerBodyElement<'data> {
    Dummy(Cow<'data, ()>),
    YourPlayerId(PlayerId),
    Thingy(ThingyNet),
    Player(PlayerNet),
}

fn serialized_size(ser: &impl Serialize) -> usize {
    postcard::serialize_with_flavor(ser, postcard::ser_flavors::Size::default()).unwrap()
}
