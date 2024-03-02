use super::gateway::EngineWatcher;

#[derive(Clone)]
pub(super) struct ServerState {
    pub gateway: EngineWatcher,
}
