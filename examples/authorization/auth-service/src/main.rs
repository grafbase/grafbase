use axum::{Json, Router, response::IntoResponse, routing::post};
use tokio::{net::TcpListener, signal};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(log_layer)
        .with(EnvFilter::new("info"))
        .init();

    let app = Router::new()
        .route("/authorized-users", post(authorized_users))
        .layer(TraceLayer::new_for_http());

    println!("Serving authorization service at 0.0.0.0:4001");
    axum::serve(TcpListener::bind("0.0.0.0:4001").await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    println!("Shutting down gracefully...");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizationResponse {
    authorized_users: Vec<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizeUserRequest {
    current_user_id: u32,
}

async fn authorized_users(
    Json(AuthorizeUserRequest { current_user_id }): Json<AuthorizeUserRequest>,
) -> impl IntoResponse {
    tracing::info!("Requesting authorized users for {current_user_id}");
    Json(AuthorizationResponse {
        authorized_users: vec![1, current_user_id],
    })
}
