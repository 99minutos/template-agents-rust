use crate::agents::orchestrator::Orchestrator;
use crate::infra::redis::RedisProvider;

pub struct AppState {
    pub orchestrator: Orchestrator,
    pub redis: RedisProvider,
}

impl AppState {
    pub fn new(orchestrator: Orchestrator, redis: RedisProvider) -> Self {
        Self { orchestrator, redis }
    }
}
