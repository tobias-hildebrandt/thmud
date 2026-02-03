use bevy::ecs::resource::Resource;
use serde::{Deserialize, Serialize};

/// Maximum serialized message size in bytes.
///
/// Based on MTU - UDP header.
// TODO: dynamically change this based on network behavior?
pub(crate) const MAX_MESSAGE_SIZE: usize = 1500 - 8;

/// Common header for all messages.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct NetHeader {
    // TODO: versioning, timing/network stuff for bandwidth calcs?
}

/// Buffer for received messages that have yet to be processed.
#[derive(Debug, Resource)]
pub(crate) struct MessageBuffer<T> {
    pub(crate) messages: Vec<T>,
}

impl<T> Default for MessageBuffer<T> {
    fn default() -> Self {
        Self {
            messages: Default::default(),
        }
    }
}

impl<T> MessageBuffer<T> {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

/// Indicates that an operation would cause the message to exceed the maximum size.
#[derive(Debug, thiserror::Error)]
#[error("Message would exceed max size of {max}B, currently {current}B")]
pub(crate) struct MessageWouldExceedMax {
    pub(crate) current: usize,
    pub(crate) max: usize,
}

/// Return the serialized size of the argument in bytes.
pub(crate) fn serialized_size(ser: &impl Serialize) -> usize {
    postcard::serialize_with_flavor(ser, postcard::ser_flavors::Size::default()).unwrap()
}
