use std::ops::Sub;

use bevy::{
    app::{FixedPreUpdate, Plugin},
    ecs::{resource::Resource, system::ResMut},
};
use serde::{Deserialize, Serialize};

pub(crate) struct GameTickPlugin;

impl Plugin for GameTickPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(GameTick::default());

        app.add_systems(FixedPreUpdate, increment_tick);
    }
}

/*
server
store global tick counter
for each net object, we must store:
    for each client:
        tick when we last sent update
        tick that the client last ACKed
prioritization filter should take into account when choosing net objects to send

client
("global" tick counter has no meaning, each object's state is in a different moment of time!)
for each net object, we must store:
    tick that we last updated the object
must ACK net object updates
*/

/// The server's (not the client's!) physics game tick counter.
#[derive(
    Debug,
    Default,
    Resource,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Hash,
)]
pub(crate) struct GameTick(u64);

impl Sub for GameTick {
    type Output = u64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

pub(crate) fn increment_tick(mut tick: ResMut<GameTick>) {
    tick.0 = tick.0.wrapping_add(1);
}
