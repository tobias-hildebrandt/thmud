use serde::{Deserialize, Serialize};

use crate::{
    networking::tick::GameTick,
    simulation::{
        player::{PlayerId, PlayerNet},
        thingy::ThingyNet,
    },
};

use super::common::{MAX_PACKET_SIZE, MessageWouldExceedMax, NetHeader, serialized_size};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerMessage {
    pub(crate) header: NetHeader,
    pub(crate) tick: GameTick,
    body_elements: Vec<ServerBodyElement>,
}

impl ServerMessage {
    pub(crate) fn new(header: NetHeader, tick: GameTick) -> Self {
        Self {
            header,
            tick,
            body_elements: Vec::new(),
        }
    }

    pub(crate) fn try_push(
        &mut self,
        element: ServerBodyElement,
    ) -> Result<(), MessageWouldExceedMax> {
        self.body_elements.push(element);

        if serialized_size(&self) > MAX_PACKET_SIZE {
            self.body_elements.pop();
            return Err(MessageWouldExceedMax {
                current: serialized_size(&self),
                max: MAX_PACKET_SIZE,
            });
        }

        Ok(())
    }

    pub(crate) fn body_elements(self) -> Vec<ServerBodyElement> {
        self.body_elements
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ServerBodyElement {
    YourPlayerId(PlayerId),
    Thingy(ThingyNet),
    Player(PlayerNet),
}
