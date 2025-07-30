use axum::{Router, routing::post};
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
        .route("/authorized-users", post(authorized_users::handler))
        .route("/policy", post(policy::handler))
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

mod authorized_users {
    use axum::{Json, response::IntoResponse};

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Response {
        authorized_users: Vec<u32>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Request {
        current_user_id: u32,
    }

    pub async fn handler(Json(Request { current_user_id }): Json<Request>) -> impl IntoResponse {
        tracing::info!("Requesting authorized users for {current_user_id}");
        Json(Response {
            authorized_users: vec![1, current_user_id],
        })
    }
}

mod policy {
    use axum::{Json, response::IntoResponse};

    #[derive(serde::Deserialize)]
    pub struct Request {
        policies: Vec<String>,
    }

    #[derive(serde::Serialize)]
    pub struct Response {
        granted: Vec<bool>,
    }

    pub async fn handler(Json(Request { policies }): Json<Request>) -> impl IntoResponse {
        tracing::info!("Requesting policy approvals for {policies:?}");
        Json(Response {
            granted: policies.iter().map(|policy| policy.starts_with("read")).collect(),
        })
    }
}
