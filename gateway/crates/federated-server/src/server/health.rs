use super::state::ServerState;
use axum::{extract::State, Json};
use http::StatusCode;

#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub(crate) enum HealthState {
    Healthy,
    Unhealthy,
}

pub(crate) async fn health(State(state): State<ServerState>) -> (StatusCode, Json<HealthState>) {
    if state.gateway().borrow().is_some() {
        (StatusCode::OK, Json(HealthState::Healthy))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(HealthState::Unhealthy))
    }
}
