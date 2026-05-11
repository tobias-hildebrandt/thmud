use crate::simulation::{
    messages::{MessageQueue, MessageToClient, MessageToServer},
    sim::Tick,
    world::WorldCells,
};

#[derive(Debug)]
pub(crate) struct Client {
    pub(crate) world: WorldCells,
    pub(crate) message_queue: MessageQueue<MessageToClient>,
}

impl Client {
    pub(crate) fn new(world_size: usize) -> Self {
        Self {
            world: WorldCells::new_default(world_size),
            message_queue: MessageQueue::new(),
        }
    }

    pub(crate) fn handle_messages(&mut self, tick: Tick) -> Vec<MessageToServer> {
        let mut messages = vec![];
        // client handle messages and sends acks
        while let Some(message) = self.message_queue.try_pop(tick) {
            let message_to_server = self.handle_single_message(message);
            // client sends acks
            messages.push(message_to_server);
        }
        messages
    }

    fn handle_single_message(&mut self, message: MessageToClient) -> MessageToServer {
        tracing::debug!("client handling message: {:?}", message);
        for update in message.updates.iter() {
            self.world.data[update.id.row][update.id.column].state = update.new_state;
        }

        let locations = message.updates.iter().map(|update| update.id).collect();

        MessageToServer {
            ack: message.tick,
            cells: locations,
        }
    }
}
