use crate::{messages::MessageToClient, messages::MessageToServer, world::WorldCells};

pub(super) fn client_handle_message(
    state: &mut WorldCells,
    message: MessageToClient,
) -> MessageToServer {
    tracing::debug!("client handling message: {:?}", message);
    for update in message.updates.iter() {
        state.data[update.id.row][update.id.column].state = update.new_state;
    }

    let locations = message.updates.iter().map(|update| update.id).collect();

    MessageToServer {
        ack: message.tick,
        cells: locations,
    }
}
