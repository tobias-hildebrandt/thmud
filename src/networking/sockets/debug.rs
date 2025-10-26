#![allow(unused)]

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct DebugNetSocket<T> {
    pub(crate) actions: VecDeque<DebugAction<T>>,
    pub(crate) wait_until: Option<Instant>,
}

impl<T> DebugNetSocket<T> {
    pub(crate) fn new() -> Self {
        Self {
            actions: VecDeque::new(),
            wait_until: Default::default(),
        }
    }

    pub(crate) fn new_with_actions(actions: impl IntoIterator<Item = DebugAction<T>>) -> Self {
        Self {
            actions: VecDeque::from_iter(actions),
            wait_until: Default::default(),
        }
    }

    pub(crate) fn next_message(&mut self) -> Option<T> {
        if self
            .wait_until
            .is_some_and(|target| target > Instant::now())
        {
            return None;
        }

        self.wait_until = None;

        match self.actions.pop_front()? {
            DebugAction::Wait(duration) => {
                self.wait_until = Some(Instant::now() + duration);
                None
            }
            DebugAction::Message(m) => Some(m),
        }
    }
}

#[derive(Debug)]
pub(crate) enum DebugAction<T> {
    Wait(Duration),
    Message(T),
}
