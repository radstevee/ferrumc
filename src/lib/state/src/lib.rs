use std::sync::Arc;
use ferrumc_ecs::Universe;

pub struct ServerState {
    pub universe: Universe
}

pub type GlobalState = Arc<ServerState>;

impl ServerState {
    pub fn new(universe: Universe) -> Self {
        Self {
            universe
        }
    }
}
