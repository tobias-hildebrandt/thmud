use crate::{
    simulation::{
        messages::{EntityUpdate, MessageToClient, MessageToServer},
        sim::Tick,
        world::{CellLocation, WorldCells, WorldData},
    },
    utils::square,
};

pub(crate) type WorldSyncState = WorldData<CellSyncState>;

#[derive(Debug, Default)]
pub(crate) enum CellSyncState {
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

impl CellSyncState {
    fn ack(&mut self, tick: Tick) {
        let mut new_state = None;

        match self {
            CellSyncState::NotSynced => {
                tracing::warn!("server received spurious ack",);
            }
            CellSyncState::Sent { sent } => {
                new_state = Some(CellSyncState::Recv {
                    sent: *sent,
                    ack: tick,
                });
            }
            CellSyncState::Recv { sent, ack: _ } => {
                new_state = Some(CellSyncState::Recv {
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
            CellSyncState::NotSynced => Self::Sent { sent: tick },
            CellSyncState::Sent { sent: _ } => Self::Sent { sent: tick },
            CellSyncState::Recv { sent: _, ack } => Self::Recv {
                sent: tick,
                ack: *ack,
            },
        };
        *self = new_state;
    }
}

pub(crate) struct PriorityCalc {
    location: CellLocation,
    priority: f32,
}

const DELAY_TARGET: usize = 10;
const DISTANCE_TARGET: f32 = 10.0;

pub(crate) fn server_handle_message(sync_states: &mut WorldSyncState, message: MessageToServer) {
    tracing::debug!("server handling message: {:?}", message);
    for location in message.cells {
        let state = &mut sync_states.data[location.row][location.column];
        state.ack(message.ack);
    }
}

// TODO: adjust priority curves
pub(crate) fn send_updates(
    current_tick: Tick,
    world: &WorldCells,
    sync_states: &mut WorldSyncState,
    num_updates: usize,
    center: &CellLocation,
) -> MessageToClient {
    let mut all_priorities: Vec<PriorityCalc> =
        Vec::with_capacity(world.world_size * world.world_size);

    for row in 0..world.world_size {
        for column in 0..world.world_size {
            let sync_state = &sync_states.data[row][column];
            let ticks_without_ack = match sync_state {
                CellSyncState::NotSynced => current_tick.0,
                CellSyncState::Sent { sent } | CellSyncState::Recv { sent, .. } => {
                    current_tick.0.saturating_sub(sent.0)
                }
            };
            let staleness_fraction = ticks_without_ack as f32 / DELAY_TARGET as f32;

            let row_distance = row as f32 - center.row as f32;
            let column_distance = column as f32 - center.column as f32;
            let distance = (square(row_distance) + square(column_distance)).sqrt();
            let distance_fraction = distance / DISTANCE_TARGET;

            let staleness_factor = 2f32.powf(staleness_fraction - 1.0);
            let distance_factor = 2f32.powf(-distance_fraction) + 0.5;

            let priority = staleness_factor * distance_factor;

            all_priorities.push(PriorityCalc {
                location: CellLocation { row, column },
                priority,
            });
        }
    }

    // sort by priorities
    all_priorities.sort_by(|first, second| second.priority.total_cmp(&first.priority));

    // TODO: print

    all_priorities.truncate(num_updates);

    // sent
    for location in all_priorities.iter().map(|prio| &prio.location) {
        sync_states.data[location.row][location.column].sent(current_tick);
    }

    let updates = all_priorities
        .into_iter()
        .map(|priority_calc| {
            let cell = &world.data[priority_calc.location.row][priority_calc.location.column];

            EntityUpdate {
                id: priority_calc.location,
                _priority: priority_calc.priority,
                new_state: cell.state,
            }
        })
        .collect();

    MessageToClient {
        tick: current_tick,
        updates,
    }
}
