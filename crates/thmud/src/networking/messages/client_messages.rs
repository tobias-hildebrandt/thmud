use serde::{Deserialize, Serialize};

use crate::{
    networking::{ecs::NetId, tick::GameTick},
    simulation::input::PlayerInput,
};

use super::common::{MAX_MESSAGE_SIZE, MessageWouldExceedMax, NetHeader, serialized_size};

/// Message sent from the client to the server.
///
/// Use [`ClientDataMessageBuilder`] for creating data messages.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientMessage {
    header: NetHeader,
    body: ClientMessageBody,
}

impl ClientMessage {
    /// Create a register message.
    pub(crate) fn register(header: NetHeader) -> Self {
        Self {
            header,
            body: ClientMessageBody::Register,
        }
    }

    /// Create an unregister message.
    pub(crate) fn unregister(header: NetHeader) -> Self {
        Self {
            header,
            body: ClientMessageBody::Unregister,
        }
    }

    /// Return the body from the message (consumes the message).
    pub(crate) fn body(self) -> ClientMessageBody {
        self.body
    }
}

/// Main contents of a [`ClientMessage`].
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ClientMessageBody {
    /// Tells the server that the client wishes to connect.
    Register,
    /// Tells the server that the client has disconnected.
    Unregister,
    /// Contains a list of client data.
    Data(ClientDataBody),
}

/// Body of a [`ClientMessage`] that contains data.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientDataBody {
    /// Client's current inputs.
    pub(crate) input: PlayerInput,
    /// List of selective net object acknowledgements.
    pub(crate) acks: Vec<NetObjAck>,
}

/// Acknowledgement from a client that it received the state of a specific network object at a
/// specific server tick.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct NetObjAck {
    pub(crate) net_id: NetId,
    pub(crate) tick: GameTick,
}

/// Helper for building a [`ClientMessageBody::Data`].
#[derive(Debug)]
pub(crate) struct ClientDataMessageBuilder {
    header: NetHeader,
    input: PlayerInput,
    acks: Vec<NetObjAck>,
}

impl ClientDataMessageBuilder {
    /// Create a new builder with a given header and player input.
    pub(crate) fn new(header: NetHeader, input: PlayerInput) -> Self {
        Self {
            header,
            input,
            acks: Vec::new(),
        }
    }

    /// Try to push a [`NetObjAck`] into the builder.
    ///
    /// # Errors
    /// Returns [`MessageWouldExceedMax`] if the ack cannot fit into the builder.
    pub(crate) fn try_add_ack(&mut self, ack: NetObjAck) -> Result<(), MessageWouldExceedMax> {
        self.acks.push(ack);

        if serialized_size(&self.clone_build()) > MAX_MESSAGE_SIZE {
            self.acks.pop();
            return Err(MessageWouldExceedMax {
                current: serialized_size(&self.clone_build()),
                max: MAX_MESSAGE_SIZE,
            });
        }

        Ok(())
    }

    /// Convert the builder into a [`ClientMessage`].
    pub(crate) fn build(self) -> ClientMessage {
        ClientMessage {
            header: self.header,
            body: ClientMessageBody::Data(ClientDataBody {
                input: self.input,
                acks: self.acks,
            }),
        }
    }

    fn clone_build(&self) -> ClientMessage {
        ClientMessage {
            header: self.header,
            body: ClientMessageBody::Data(ClientDataBody {
                input: self.input,
                // TODO: convert to Cow to prevent clone during size check
                acks: self.acks.clone(),
            }),
        }
    }
}
