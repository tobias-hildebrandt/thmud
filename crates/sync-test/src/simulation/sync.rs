use crate::simulation::{
    client::Client,
    server::Server,
    world::{CellLocation, WorldData},
};

#[derive(Debug, Default)]
pub(crate) struct InFlightSyncStatus {
    pub(crate) same: bool,
    pub(crate) update_in_flight: InFlight,
    // TODO: track ACKs too
    // ack_in_flight: InFlight,
    // pub(crate) server_sync_state: SimpleServerCellSyncState,
}

#[derive(Debug, Default)]
pub(crate) struct InFlight {
    pub(crate) old: bool,
    pub(crate) current: bool,
}

pub(crate) type WorldInFlightSyncStatus = WorldData<InFlightSyncStatus>;

impl WorldInFlightSyncStatus {
    pub(crate) fn update(&mut self, server: &Server, client: &Client) {
        let world_size = self.world_size();

        for row in 0..world_size {
            for column in 0..world_size {
                let location = CellLocation { row, column };

                let server_val = server.world.data[row][column].state;
                let client_val = client.world.data[row][column].state;

                let same_values = server_val == client_val;

                let old_update_in_flight = client.message_queue.iter().any(|update| {
                    update
                        .message
                        .updates
                        .iter()
                        .any(|update| update.id == location && update.new_state != server_val)
                });
                let current_update_in_flight = client.message_queue.iter().any(|update| {
                    update
                        .message
                        .updates
                        .iter()
                        .any(|update| update.id == location && update.new_state == server_val)
                });

                let status = InFlightSyncStatus {
                    same: same_values,
                    update_in_flight: InFlight {
                        old: old_update_in_flight,
                        current: current_update_in_flight,
                    },
                };

                self.data[row][column] = status;
            }
        }
    }
}
