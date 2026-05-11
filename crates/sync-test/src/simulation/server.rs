use crate::simulation::{
    messages::{EntityUpdate, MessageQueue, MessageToClient, MessageToServer},
    priority::{Priority, WorldPriority},
    sim::Tick,
    world::{CellLocation, WorldCells, WorldData},
};

// TODO: add last_changed_tick WorldData
#[derive(Debug)]
pub(crate) struct Server {
    pub(crate) world: WorldCells,
    pub(crate) sync_states: ServerSyncState,
    pub(crate) priorities: WorldPriority,
    pub(crate) message_queue: MessageQueue<MessageToServer>,
}

impl Server {
    pub(crate) fn new_random(world_size: usize) -> Self {
        Self {
            world: WorldCells::new_random(world_size),
            sync_states: ServerSyncState::new_default(world_size),
            priorities: WorldPriority::new_default(world_size),
            message_queue: MessageQueue::new(),
        }
    }
}

pub(crate) type ServerSyncState = WorldData<ServerCellSyncState>;

#[derive(Debug, Default)]
pub(crate) enum ServerCellSyncState {
    #[default]
    NotSynced,
    Sent {
        sent: Tick,
    },
    Recv {
        sent: Tick,
        ack: Tick,
    },
}

impl ServerCellSyncState {
    fn ack(&mut self, tick: Tick) {
        let mut new_state = None;

        match self {
            ServerCellSyncState::NotSynced => {
                tracing::warn!("server received spurious ack",);
            }
            ServerCellSyncState::Sent { sent } => {
                new_state = Some(ServerCellSyncState::Recv {
                    sent: *sent,
                    ack: tick,
                });
            }
            ServerCellSyncState::Recv { sent, ack: _ } => {
                new_state = Some(ServerCellSyncState::Recv {
                    sent: *sent,
                    ack: tick,
                })
            }
        }

        if let Some(new_state) = new_state {
            *self = new_state;
        }
    }

    fn sent(&mut self, tick: Tick) {
        let new_state = match self {
            ServerCellSyncState::NotSynced => Self::Sent { sent: tick },
            ServerCellSyncState::Sent { sent: _ } => Self::Sent { sent: tick },
            ServerCellSyncState::Recv { sent: _, ack } => Self::Recv {
                sent: tick,
                ack: *ack,
            },
        };
        *self = new_state;
    }
}

impl Server {
    /// Randomly mutate server world.
    pub(crate) fn mutate(&mut self, mutations: u128) {
        tracing::debug!("mutations this tick: {}", mutations);
        for _ in 0..mutations {
            self.world.random_mutation();
        }
    }

    pub(crate) fn handle_messages(&mut self, tick: Tick) {
        while let Some(message) = self.message_queue.try_pop(tick) {
            self.handle_single_message(message);
        }
    }

    fn handle_single_message(&mut self, message: MessageToServer) {
        tracing::debug!("server handling message: {:?}", message);
        for location in message.cells {
            let state = &mut self.sync_states.data[location.row][location.column];
            state.ack(message.ack);
        }
    }

    // TODO: adjust priority curves
    pub(crate) fn calculate_priorities(&mut self, tick: Tick, center: &CellLocation) {
        let world_size = self.world.world_size();
        for row in 0..world_size {
            for column in 0..world_size {
                let sync_state = &self.sync_states.data[row][column];
                self.priorities.data[row][column] =
                    Priority::calculate(tick, CellLocation { row, column }, sync_state, center);
            }
        }
    }

    pub(crate) fn send_updates(&mut self, tick: Tick, num_updates: usize) -> MessageToClient {
        let mut all_priorities = self.priorities.cell_priorities().collect::<Vec<_>>();

        // sort by priorities
        all_priorities.sort_by(|first, second| second.priority.0.total_cmp(&first.priority.0));

        // TODO: print

        all_priorities.truncate(num_updates);

        // update sync states for the packets we are sending
        for location in all_priorities.iter().map(|prio| &prio.location) {
            self.sync_states.data[location.row][location.column].sent(tick);
        }

        // add cell state to sent updates
        let updates = all_priorities
            .into_iter()
            .map(|priority_calc| {
                let cell =
                    &self.world.data[priority_calc.location.row][priority_calc.location.column];

                EntityUpdate {
                    id: priority_calc.location,
                    priority: priority_calc.priority,
                    new_state: cell.state,
                }
            })
            .collect();

        MessageToClient { tick, updates }
    }
}
