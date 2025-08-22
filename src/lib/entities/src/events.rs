use bevy_ecs::prelude::Event;
use ferrumc_core::transform::position::Position;

use crate::EntityType;

#[derive(Clone, Event)]
pub struct SpawnEntityEvent {
    pub entity_kind: EntityType,
    pub position: Position,
}
