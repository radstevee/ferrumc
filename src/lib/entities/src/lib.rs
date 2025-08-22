use bevy_ecs::prelude::*;
use ferrumc_macros::Deref;

pub mod bundles;
pub mod components;
pub mod events;
pub mod spawner;
pub mod factory;

/// The health of an entity.
#[derive(Clone, Copy, Debug, Component, Deref)]
pub struct Health(f64);

/// The maximum health of an entity.
#[derive(Clone, Copy, Debug, Component, Deref)]
pub struct MaxHealth(f64);

/// The type of an entity.
#[derive(Clone, Copy, Debug, Component, Deref)]
pub struct EntityType {
    /// The id of the entity type. (Found in the registry under minecraft:entity_type)
    pub id: u64,
}

impl From<u64> for EntityType {
    fn from(value: u64) -> Self {
        Self { id: value }
    }
}
