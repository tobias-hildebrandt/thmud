use std::collections::VecDeque;

use crate::simulation::{sim::Tick, world::CellLocation};

/// A whole-state update.
#[derive(Debug)]
pub(crate) struct Update<Id, State> {
    /// What tick was this update sent.
    pub(crate) tick: Tick,
    /// List of state updates.
    pub(crate) updates: Vec<EntityUpdate<Id, State>>,
}

#[derive(Debug)]
pub(crate) struct EntityUpdate<Id, State> {
    pub(crate) id: Id,
    pub(crate) _priority: f32,
    pub(crate) new_state: State,
}

#[derive(Debug)]
pub(crate) struct MessageQueue<Message>(VecDeque<TickMessage<Message>>);

impl<Message> MessageQueue<Message> {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Pop a message if it's time to deliver it.
    pub(crate) fn try_pop(&mut self, current_tick: Tick) -> Option<Message> {
        if self
            .0
            .front()
            .is_some_and(|message| message.tick_to_arrive <= current_tick)
        {
            // SAFETY: unwrap OK since we just checked with get()
            Some(self.0.pop_front().unwrap().message)
        } else {
            None
        }
    }

    /// Push a message to be delivered at a specific tick.
    pub(crate) fn push(&mut self, message: Message, tick: Tick) {
        self.0.push_back(TickMessage {
            tick_to_arrive: tick,
            message,
        });

        // sort
        self.0
            .make_contiguous()
            .sort_by(|first, second| first.tick_to_arrive.cmp(&second.tick_to_arrive));
    }
}

impl<Message> Default for MessageQueue<Message> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub(crate) type MessageToClient = Update<CellLocation, u8>;

#[derive(Debug)]
pub struct MessageToServer {
    pub(crate) ack: Tick,
    pub(crate) cells: Vec<CellLocation>,
}

/// Mock "messages in flight" that arrive at a specific tick.
#[derive(Debug)]
pub(crate) struct TickMessage<Message> {
    pub(crate) tick_to_arrive: Tick,
    pub(crate) message: Message,
}
