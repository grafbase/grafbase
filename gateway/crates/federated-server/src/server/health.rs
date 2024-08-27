use std::net::SocketAddr;

use gateway_config::{HealthConfig, TlsConfig};

use super::{state::ServerState, ServerRuntime};
use axum::{extract::State, routing::get, Json, Router};
use grafbase_telemetry::span::GRAFBASE_TARGET;
use http::StatusCode;

#[derive(Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub(crate) enum HealthState {
    Healthy,
    Unhealthy,
}

pub(crate) async fn health<SR>(State(state): State<ServerState<SR>>) -> (StatusCode, Json<HealthState>) {
    if state.gateway.borrow().is_some() {
        (StatusCode::OK, Json(HealthState::Healthy))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(HealthState::Unhealthy))
    }
}

pub(super) async fn bind_health_endpoint<SR: ServerRuntime>(
    addr: SocketAddr,
    tls_config: Option<TlsConfig>,
    health_config: HealthConfig,
    state: ServerState<SR>,
) -> crate::Result<()> {
    let scheme = if tls_config.is_some() { "https" } else { "http" };
    let path = &health_config.path;
    let app = Router::new()
        .route(path, get(health))
        .with_state(state)
        .into_make_service();

    tracing::info!(target: GRAFBASE_TARGET, "Health check endpoint exposed at {scheme}://{addr}{path}");

    match tls_config {
        Some(tls) => {
            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?;
        }
        None => axum_server::bind(addr).serve(app).await.map_err(crate::Error::Server)?,
    }

    Ok(())
}
