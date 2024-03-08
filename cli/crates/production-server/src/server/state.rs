use super::gateway::GatewayWatcher;

#[derive(Clone)]
pub(super) struct ServerState {
    pub gateway: GatewayWatcher,
}
