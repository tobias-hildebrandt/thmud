use std::{fmt::Display, ops::Range, str::FromStr};

use anyhow::Context;
use clap::Parser;

use crate::simulation::world::CellLocation;

/// Simulation config.
#[derive(Debug, Parser)]
pub struct SyncTestConfig {
    /// Size of world (one side of the square).
    #[arg(long, default_value_t = Self::default_world_size())]
    pub world_size: usize,
    /// Simulated packet latency (static or range).
    #[arg(long, default_value_t = Self::default_latency())]
    pub latency: StaticOrRandom,
    /// Server world mutations per tick (static or range).
    #[arg(long, default_value_t = Self::default_mutations_per_tick())]
    pub mutations_per_tick: StaticOrRandom,
    /// Number of sync-updates per packet.
    #[arg(long, default_value_t = Self::default_num_sync_updates())]
    pub num_sync_updates: usize,
    /// Coordinates of center cell (for distance priority calculations).
    #[arg(long, default_value_t = Self::default_center_cell())]
    pub center: CenterCell,
}

impl SyncTestConfig {
    fn default_world_size() -> usize {
        16
    }

    fn default_latency() -> StaticOrRandom {
        StaticOrRandom::RandomRange(1..4)
    }

    fn default_mutations_per_tick() -> StaticOrRandom {
        StaticOrRandom::RandomRange(5..10)
    }

    fn default_num_sync_updates() -> usize {
        10
    }

    fn default_center_cell() -> CenterCell {
        CenterCell(CellLocation { row: 0, column: 0 })
    }
}

impl Default for SyncTestConfig {
    fn default() -> Self {
        Self {
            world_size: Self::default_world_size(),
            latency: Self::default_latency(),
            mutations_per_tick: Self::default_mutations_per_tick(),
            num_sync_updates: Self::default_num_sync_updates(),
            center: Self::default_center_cell(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StaticOrRandom {
    Static(u128),
    RandomRange(Range<u128>),
}

impl StaticOrRandom {
    pub(crate) fn get(&self) -> u128 {
        match self {
            StaticOrRandom::Static(l) => *l,
            StaticOrRandom::RandomRange(range) => rand::random_range(range.clone()),
        }
    }
}

impl FromStr for StaticOrRandom {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("..") {
            let mut split = s.split("..");
            let first = split
                .next()
                .context("range split on .. has no first element")?;
            let second = split
                .next()
                .context("range split on .. has no second element")?;
            let first = first.parse()?;
            let second = second.parse()?;
            Ok(Self::RandomRange(first..second))
        } else {
            Ok(Self::Static(s.parse()?))
        }
    }
}

impl Display for StaticOrRandom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaticOrRandom::Static(num) => write!(f, "{num}"),
            StaticOrRandom::RandomRange(range) => {
                write!(f, "{}..{}", range.start, range.end)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CenterCell(pub CellLocation);

impl FromStr for CenterCell {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut split = s.split(",");
        let first = split.next().context("no row specified")?;
        let second = split.next().context("nothing after row")?;
        Ok(CenterCell(CellLocation {
            row: first.parse()?,
            column: second.parse()?,
        }))
    }
}

impl Display for CenterCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0.row, self.0.column)
    }
}
