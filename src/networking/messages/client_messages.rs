use serde::{Deserialize, Serialize};

use crate::{
    networking::{ecs::NetId, tick::GameTick},
    simulation::input::PlayerInput,
};

use super::common::{MAX_PACKET_SIZE, MessageWouldExceedMax, NetHeader, serialized_size};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientMessage {
    header: NetHeader,
    body: ClientMessageBody,
}

impl ClientMessage {
    pub(crate) fn register(header: NetHeader) -> Self {
        Self {
            header,
            body: ClientMessageBody::Register,
        }
    }

    pub(crate) fn unregister(header: NetHeader) -> Self {
        Self {
            header,
            body: ClientMessageBody::Unregister,
        }
    }

    pub(crate) fn body(self) -> ClientMessageBody {
        self.body
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ClientMessageBody {
    Register,
    Unregister,
    Data(ClientDataBody),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientDataBody {
    pub(crate) input: PlayerInput,
    pub(crate) acks: Vec<NetObjAck>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]

pub(crate) struct NetObjAck {
    pub(crate) net_id: NetId,
    pub(crate) tick: GameTick,
}

#[derive(Debug)]
pub(crate) struct ClientDataMessageBuilder {
    header: NetHeader,
    input: PlayerInput,
    acks: Vec<NetObjAck>,
}

impl ClientDataMessageBuilder {
    pub(crate) fn new(header: NetHeader, input: PlayerInput) -> Self {
        Self {
            header,
            input,
            acks: Vec::new(),
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

    pub(crate) fn build(self) -> ClientMessage {
        ClientMessage {
            header: self.header,
            body: ClientMessageBody::Data(ClientDataBody {
                input: self.input,
                acks: self.acks,
            }),
        }
    }

    pub(crate) fn try_add_ack(&mut self, ack: NetObjAck) -> Result<(), MessageWouldExceedMax> {
        self.acks.push(ack);

        if serialized_size(&self.clone_build()) > MAX_PACKET_SIZE {
            self.acks.pop();
            return Err(MessageWouldExceedMax {
                current: serialized_size(&self.clone_build()),
                max: MAX_PACKET_SIZE,
            });
        }

        Ok(())
    }
}
