use crate::simulation::{
    client::client_handle_message,
    config::SyncTestConfig,
    messages::{MessageQueue, MessageToClient, MessageToServer},
    server::{WorldSyncState, send_updates, server_handle_message},
    world::{TwoWorldDisplay, WorldCells},
};

/// Time in ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tick(pub(crate) u128);

// TODO: split server+client(s)
#[derive(Debug)]
pub(crate) struct Sim {
    pub(crate) config: SyncTestConfig,
    pub(crate) tick: Tick,

    // server
    pub(crate) server_world: WorldCells,
    pub(crate) sync_states: WorldSyncState,
    pub(crate) server_queue: MessageQueue<MessageToServer>,

    // client
    pub(crate) client_world: WorldCells,
    pub(crate) client_queue: MessageQueue<MessageToClient>,
}

impl Sim {
    pub fn new(config: SyncTestConfig) -> Self {
        let server_world = WorldCells::new_random(config.world_size);
        let sync_states = WorldSyncState::new_default(config.world_size);
        let server_queue = MessageQueue::<MessageToServer>::new();

        let client_world = WorldCells::new_default(config.world_size);
        let client_queue = MessageQueue::<MessageToClient>::new();

        let tick: Tick = Tick(0);

        Self {
            config,
            tick,
            server_world,
            sync_states,
            server_queue,
            client_world,
            client_queue,
        }
    }

    pub fn tick(&mut self) {
        tracing::info!("tick {:?}", self.tick);

        // mutate server world
        let mutations = self.config.mutations_per_tick.get();
        tracing::debug!("mutations this tick: {}", mutations);
        for _ in 0..mutations {
            self.server_world.random_mutation();
        }

        // server handle messages
        while let Some(message) = self.server_queue.try_pop(self.tick) {
            server_handle_message(&mut self.sync_states, message);
        }

        // server send changes
        {
            let message_to_client = send_updates(
                self.tick,
                &self.server_world,
                &mut self.sync_states,
                self.config.num_sync_updates,
                &self.config.center.0,
            );
            let updates_str = message_to_client
                .updates
                .iter()
                .fold(String::new(), |accum, next| {
                    format!("{}, {:?}", accum, next)
                });
            tracing::debug!("server sending updates: {}", updates_str);
            self.client_queue.push(
                message_to_client,
                Tick(self.tick.0 + self.config.latency.get()),
            );
        }

        // client handle messages and sends acks
        while let Some(message) = self.client_queue.try_pop(self.tick) {
            let message_to_server = client_handle_message(&mut self.client_world, message);
            // client sends acks
            self.server_queue.push(
                message_to_server,
                Tick(self.tick.0 + self.config.latency.get()),
            );
        }

        // print states
        // tracing::info!("server:\n{}\nclient:\n{}", server_world, client_world);
        tracing::info!(
            "\n{}",
            TwoWorldDisplay {
                server: &self.server_world,
                client: &self.client_world
            }
        );
        tracing::debug!("messages in flight to server: {:?}", self.server_queue);
        tracing::debug!("messages in flight to client: {:?}", self.client_queue);

        // compare states, accumulate stats

        self.tick.0 += 1;
    }
}
