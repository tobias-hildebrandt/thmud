use crate::{
    simulation::{
        server::ServerCellSyncState,
        sim::Tick,
        world::{CellLocation, WorldData},
    },
    utils::square,
};

const DELAY_TARGET: usize = 10;
const DISTANCE_TARGET: f32 = 20.0;

/// Priority.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Priority(pub(crate) f32);

pub(crate) type WorldPriority = WorldData<Priority>;

impl WorldPriority {
    pub(crate) fn cell_priorities(&self) -> impl Iterator<Item = CellPriority> {
        self.data.iter().enumerate().flat_map(|(row_num, row)| {
            row.iter()
                .enumerate()
                .map(move |(column_num, cell)| CellPriority {
                    location: CellLocation {
                        row: row_num,
                        column: column_num,
                    },
                    priority: *cell,
                })
        })
    }
}

/// [`Priority`] with the [`CellLocation`].
#[derive(Debug)]
pub(crate) struct CellPriority {
    pub(crate) location: CellLocation,
    pub(crate) priority: Priority,
}

impl Priority {
    pub(crate) fn calculate(
        tick: Tick,
        location: CellLocation,
        sync_state: &ServerCellSyncState,
        center: &CellLocation,
    ) -> Self {
        let ticks_without_ack = match sync_state {
            ServerCellSyncState::NotSynced => tick.0,
            ServerCellSyncState::Sent { sent } | ServerCellSyncState::Recv { sent, .. } => {
                tick.0.saturating_sub(sent.0)
            }
        };
        let staleness_fraction = ticks_without_ack as f32 / DELAY_TARGET as f32;

        let row_distance = location.row as f32 - center.row as f32;
        let column_distance = location.column as f32 - center.column as f32;
        let distance = (square(row_distance) + square(column_distance)).sqrt();
        let distance_fraction = distance / DISTANCE_TARGET;

        let staleness_factor = 2f32.powf(staleness_fraction - 1.0);
        let distance_factor = 2f32.powf(-distance_fraction) + 0.5;

        Priority(staleness_factor * distance_factor)
    }
}
