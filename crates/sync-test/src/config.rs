use std::{io::Read, ops::Range, time::Duration};

#[derive(Debug)]
pub struct SyncTestConfig {
    pub world_size: usize,
    pub wait_for_tick: WaitForTick,
    pub latency: StaticOrRandom,
    pub mutations_per_tick: StaticOrRandom,
    pub num_sync_updates: usize,
}

impl Default for SyncTestConfig {
    fn default() -> Self {
        Self {
            world_size: 16,
            wait_for_tick: WaitForTick::StdinNewline,
            latency: StaticOrRandom::Static(3),
            mutations_per_tick: StaticOrRandom::Static(10),
            num_sync_updates: 10,
        }
    }
}

#[derive(Debug)]
pub enum StaticOrRandom {
    Static(u128),
    RandomRange(Range<u128>),
}

impl StaticOrRandom {
    pub(super) fn get(&self) -> u128 {
        match self {
            StaticOrRandom::Static(l) => *l,
            StaticOrRandom::RandomRange(range) => rand::random_range(range.clone()),
        }
    }
}

#[derive(Debug)]

pub enum WaitForTick {
    Sleep(Duration),
    StdinNewline,
}

impl WaitForTick {
    pub(super) fn wait(&self) {
        match self {
            WaitForTick::Sleep(duration) => std::thread::sleep(*duration),
            WaitForTick::StdinNewline => {
                let mut read_char = [0u8];
                let mut read = std::io::stdin().lock();
                loop {
                    let bytes_read = read
                        .read(&mut read_char)
                        .expect("unable to read from stdin");
                    if bytes_read == 0 {
                        continue;
                    }
                    break;
                }
            }
        }
    }
}
