use bevy::{
    app::{FixedPreUpdate, Plugin},
    ecs::{
        event::{Event, EventWriter},
        resource::Resource,
        system::ResMut,
    },
};
use tracing::info;

pub(crate) struct GameNetRatePlugin {
    /// Rate per fixed-step.
    rate: u32,
}

impl GameNetRatePlugin {
    /// Once every other fixed-step.
    const DEFAULT_RATE: u32 = 2;

    pub(crate) fn new(rate: u32) -> Self {
        Self { rate }
    }
}

impl Default for GameNetRatePlugin {
    fn default() -> Self {
        Self {
            rate: GameNetRatePlugin::DEFAULT_RATE,
        }
    }
}

impl Plugin for GameNetRatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(NetRateCounter::new(self.rate));
        app.add_event::<NetTick>();

        app.add_systems(FixedPreUpdate, tick_net);
    }
}

/// Event for whenever the netcode should trigger.
#[derive(Debug, Event)]
pub(crate) struct NetTick;

#[derive(Debug, Resource)]
struct NetRateCounter {
    counter: u32,
    max: u32,
}

impl NetRateCounter {
    fn new(rate: u32) -> Self {
        Self {
            counter: 0,
            max: rate,
        }
    }
}

fn tick_net(mut timer: ResMut<NetRateCounter>, mut writer: EventWriter<NetTick>) {
    timer.counter += 1;
    if timer.counter == timer.max {
        timer.counter = 0;
        info!("net tick");
        writer.write(NetTick);
    }
}
