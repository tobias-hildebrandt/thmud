use std::marker::PhantomData;

use bevy::{ecs::component::Component, math::Vec2};
use bevy_rapier2d::prelude::ExternalForce;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};

use super::ecs::Networked;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ExternalForceSerde {
    pub(crate) force: Vec2,
    pub(crate) torque: f32,
}

impl From<ExternalForceSerde> for ExternalForce {
    fn from(value: ExternalForceSerde) -> Self {
        Self {
            force: value.force,
            torque: value.torque,
        }
    }
}

impl From<ExternalForce> for ExternalForceSerde {
    fn from(value: ExternalForce) -> Self {
        Self {
            force: value.force,
            torque: value.torque,
        }
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum ColliderMassPropertiesSerde {
//     Density(f32),
//     Mass(f32),
// }

// impl From<ColliderMassPropertiesSerde> for ColliderMassProperties {
//     fn from(value: ColliderMassPropertiesSerde) -> Self {
//         match value {
//             ColliderMassPropertiesSerde::Density(d) => Self::Density(d),
//             ColliderMassPropertiesSerde::Mass(m) => Self::Mass(m),
//         }
//     }
// }

// impl From<ColliderMassProperties> for ColliderMassPropertiesSerde {
//     fn from(value: ColliderMassProperties) -> Self {
//         match value {
//             ColliderMassProperties::Density(d) => Self::Density(d),
//             ColliderMassProperties::Mass(m) => Self::Mass(m),
//             ColliderMassProperties::MassProperties(_) => {
//                 panic!("custom mass properties unsupported");
//             }
//         }
//     }
// }

pub(crate) struct FromIntoNetworked<U>(PhantomData<U>);

// U: ExternalForceSerde
// T: ExternalForce

impl<'de, T: Component, U> DeserializeAs<'de, Networked<T>> for FromIntoNetworked<U>
where
    U: Into<T>,
    U: Deserialize<'de>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Networked<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Networked(U::deserialize(deserializer)?.into()))
    }
}

impl<T: Component, U> SerializeAs<Networked<T>> for FromIntoNetworked<U>
where
    T: Into<U> + Clone,
    U: Serialize,
{
    fn serialize_as<S>(source: &Networked<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source.clone().0.into().serialize(serializer)
    }
}
