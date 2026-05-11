use crate::simulation::{
    client::Client, config::SimConfig, server::Server, sync::WorldInFlightSyncStatus,
    world::TwoWorldDisplay,
};

/// Time in ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tick(pub(crate) u128);

// TODO: split server+client(s)
#[derive(Debug)]
pub(crate) struct Sim {
    pub(crate) config: SimConfig,
    pub(crate) tick: Tick,

    // server
    pub(crate) server: Server,
    // client
    pub(crate) client: Client,

    // calculated
    pub(crate) sync_status: WorldInFlightSyncStatus,
}

impl Sim {
    pub fn new(config: SimConfig) -> Self {
        let server = Server::new_random(config.world_size);

        let client = Client::new(config.world_size);

        let sync_status = WorldInFlightSyncStatus::new_default(config.world_size);

        let tick: Tick = Tick(0);

        Self {
            config,
            tick,
            server,
            client,
            sync_status,
        }
    }

    pub fn tick(&mut self) {
        tracing::info!("tick {:?}", self.tick);

        // server handles messages
        self.server.handle_messages(self.tick);

        // mutate server
        self.server.mutate(self.config.mutations_per_tick.get());

        // calculate priorities
        self.server
            .calculate_priorities(self.tick, &self.config.center.0);

        // server sends a message
        let server_message = self
            .server
            .send_updates(self.tick, self.config.num_sync_updates);
        self.client.message_queue.push(
            server_message,
            Tick(self.tick.0 + self.config.latency.get()),
        );

        // client handle messages and sends acks
        for message in self.client.handle_messages(self.tick) {
            self.server
                .message_queue
                .push(message, Tick(self.tick.0 + self.config.latency.get()));
        }

        // print states
        tracing::info!(
            "\n{}",
            TwoWorldDisplay {
                server: &self.server.world,
                client: &self.client.world,
            }
        );
        tracing::debug!(
            "messages in flight to server: {:?}",
            self.server.message_queue
        );
        tracing::debug!(
            "messages in flight to client: {:?}",
            self.client.message_queue
        );

        // compare states
        self.sync_status.update(&self.server, &self.client);

        // TODO: track statistics

        self.tick.0 += 1;
    }
}
