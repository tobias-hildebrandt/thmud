use std::cmp::Ordering;
use std::fmt::Debug;

use bevy::math::{Vec3, Vec3Swizzles};

use crate::simulation::player::PlayerId;

use super::{ecs::NetObj, server::ClientNetObjStateMap};

// TODO: server-side analytics
// TODO: cache priorities to avoid O(n log n) * num_players each frame (n is # of net objs)
pub(super) fn net_obj_priority(
    player_id: PlayerId,
    player_coords: Vec3,
    net_obj: &NetObj,
    client_state_map: &ClientNetObjStateMap,
) -> Priority {
    // TODO: set dynamically based on network conditions
    const TARGET_LATEST_DELAY: u64 = 5;

    // TODO: adjust? set client side to despawn past this limit?
    const MAX_DISTANCE: f32 = 5_000.0;

    let client_state = client_state_map.0.get(&net_obj.net_id());

    let staleness_multiplier = match client_state.and_then(|state| state.ticks_since_ack()) {
        Some(delay) => IntegralOrDefault::Integral {
            num: IntegralNumber {
                numer: delay,
                denom: TARGET_LATEST_DELAY,
            },
            // TODO: tweak function -- exponential? or quadratic?
            func: |num: IntegralNumber<u64>| (num.numer as f32 + 1.0) / (num.denom as f32 / 2.0),
        },
        None => IntegralOrDefault::Default(1.0),
    };

    let distance_multiplier = match net_obj
        .coords()
        .map(|net_coords| net_coords.xy().distance(player_coords.xy()))
    {
        Some(distance) => IntegralOrDefault::Integral {
            num: IntegralNumber {
                numer: distance,
                denom: MAX_DISTANCE,
            },
            // TODO: tweak function
            func: |num: IntegralNumber<f32>| 1.0 - (num.numer / num.denom).powi(2),
        },
        None => IntegralOrDefault::Default(1.0),
    };

    match net_obj {
        NetObj::Player(player_net) => {
            // TODO: move to associated function/constant
            let type_priority = 0.8;
            if player_net.player_id.0 == player_id {
                Priority::Override // MUST be included
            } else {
                Priority::Normal {
                    staleness: staleness_multiplier,
                    distance: distance_multiplier,
                    typ: type_priority,
                }
            }
        }
        NetObj::Thingy(_thingy_net) => {
            let type_priority = 0.7;
            Priority::Normal {
                staleness: staleness_multiplier,
                distance: distance_multiplier,
                typ: type_priority,
            }
        }
    }
}

pub(crate) enum IntegralOrDefault<T> {
    Integral {
        num: IntegralNumber<T>,
        func: fn(IntegralNumber<T>) -> f32,
    },
    Default(f32),
}

impl<T: Copy> IntegralOrDefault<T> {
    fn as_f32(&self) -> f32 {
        match self {
            IntegralOrDefault::Integral { num, func } => func(*num),
            IntegralOrDefault::Default(n) => *n,
        }
    }
}

impl<T: Debug> Debug for IntegralOrDefault<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integral { num, func: _ } => {
                f.debug_struct("Integral").field("num", num).finish()
            }
            Self::Default(arg0) => f.debug_tuple("Default").field(arg0).finish(),
        }
    }
}

impl<T: PartialEq + Copy> PartialEq for IntegralOrDefault<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_f32() == other.as_f32()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IntegralNumber<T> {
    numer: T,
    denom: T,
}

impl<T: PartialEq> PartialEq for IntegralNumber<T> {
    fn eq(&self, other: &Self) -> bool {
        self.numer.eq(&other.numer) && self.denom.eq(&other.denom)
    }
}

#[derive(PartialEq)]
pub(super) enum Priority {
    Override,
    Normal {
        staleness: IntegralOrDefault<u64>,
        distance: IntegralOrDefault<f32>,
        typ: f32,
    },
}

impl Priority {
    fn as_f32(&self) -> f32 {
        match self {
            Priority::Override => 1.0,
            Priority::Normal {
                staleness,
                distance,
                typ,
            } => (staleness.as_f32()) * (distance.as_f32()) * *typ,
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_f32().total_cmp(&other.as_f32()))
    }
}

impl Debug for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let float = self.as_f32();

        write!(f, "{float:>3.3}(")?;

        match self {
            Self::Override => {
                write!(f, "Override")?;
            }
            Self::Normal {
                staleness,
                distance,
                typ,
            } => {
                f.debug_struct("Normal")
                    .field("staleness", staleness)
                    .field("staleness.as_f32()", &staleness.as_f32())
                    .field("distance", distance)
                    .field("distance.as_f32()", &distance.as_f32())
                    .field("typ", typ)
                    .finish()?;
            }
        }
        write!(f, ")")
    }
}
