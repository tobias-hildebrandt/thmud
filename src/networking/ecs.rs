use bevy::{
    app::Plugin,
    ecs::{bundle::Bundle, component::Component},
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Clone, Copy, Default)]
pub(crate) struct Networked<T: Component> {
    pub(crate) component: Option<T>,
}

impl<T: Component> Networked<T> {
    pub(crate) const fn none() -> Self {
        Self { component: None }
    }

    pub(crate) const fn new(component: T) -> Self {
        Self {
            component: Some(component),
        }
    }
}

impl<T: Component> From<T> for Networked<T> {
    fn from(value: T) -> Self {
        Self {
            component: Some(value),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NetId(pub(crate) u128);

#[derive(Debug, Bundle)]
pub(crate) struct NetPhysicsObjectBundle {
    pub(crate) id: NetId,
    pub(crate) transform: Networked<Transform>,
}
