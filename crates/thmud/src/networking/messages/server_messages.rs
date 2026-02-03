use serde::{Deserialize, Serialize};

use crate::{
    networking::{ecs::NetObj, tick::GameTick},
    simulation::player::PlayerId,
};

use super::common::{MAX_MESSAGE_SIZE, MessageWouldExceedMax, NetHeader, serialized_size};

/// Message sent from the client to the server.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerMessage {
    pub(crate) header: NetHeader,
    pub(crate) tick: GameTick,
    body_elements: Vec<ServerBodyElement>,
}

impl ServerMessage {
    /// Create a new message with the given header and tick.
    pub(crate) fn new(header: NetHeader, tick: GameTick) -> Self {
        Self {
            header,
            tick,
            body_elements: Vec::new(),
        }
    }

    /// Try to push a body element into the message.
    ///
    /// # Errors
    /// Returns [`MessageWouldExceedMax`] if the element cannot fit into the message.
    pub(crate) fn try_push(
        &mut self,
        element: ServerBodyElement,
    ) -> Result<(), MessageWouldExceedMax> {
        self.body_elements.push(element);

        if serialized_size(&self) > MAX_MESSAGE_SIZE {
            self.body_elements.pop();
            return Err(MessageWouldExceedMax {
                current: serialized_size(&self),
                max: MAX_MESSAGE_SIZE,
            });
        }

        Ok(())
    }

    /// Return the body elements from the message (consumes the message).
    pub(crate) fn body_elements(self) -> Vec<ServerBodyElement> {
        self.body_elements
    }
}

/// Elements that make up the body of a server message.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerBodyElement {
    /// Tells a peer what their [`PlayerId`] is.
    YourPlayerId(PlayerId),
    /// Sends the current state of a [`NetObj`].
    NetObj(NetObj),
}
