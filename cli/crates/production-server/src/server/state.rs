use super::gateway::GatewayWatcher;
use axum::response::Html;

#[derive(Clone)]
pub(super) struct ServerState {
    pub pathfinder_html: Html<String>,
    pub gateway: GatewayWatcher,
}
