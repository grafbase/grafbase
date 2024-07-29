use axum::{response::IntoResponse, routing::post, Json, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(log_layer)
        .with(EnvFilter::new("debug"))
        .init();

    let app = Router::new()
        .route("/authorize-user", post(authorize_user))
        .route("/authorize-address", post(authorize_address))
        .layer(TraceLayer::new_for_http());

    println!("Serving authorization service at 127.0.0.1:4001");
    axum::serve(TcpListener::bind("127.0.0.1:4001").await?, app).await?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizationResponse {
    authorized: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizeUserRequest {
    current_user_id: usize,
    user_id: usize,
}

async fn authorize_user(
    Json(AuthorizeUserRequest {
        current_user_id,
        user_id,
    }): Json<AuthorizeUserRequest>,
) -> impl IntoResponse {
    let is_authorized = user_id <= current_user_id;
    tracing::info!(
        "Authorizing access to user {} for user {}: {is_authorized}",
        user_id,
        current_user_id
    );
    Json(AuthorizationResponse {
        authorized: is_authorized,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizeAddressRequest {
    current_user_id: usize,
    owner_id: usize,
}

async fn authorize_address(
    Json(AuthorizeAddressRequest {
        current_user_id,
        owner_id,
    }): Json<AuthorizeAddressRequest>,
) -> impl IntoResponse {
    let is_authorized = owner_id == current_user_id;
    tracing::info!(
        "Authorizing access to address of user {} for user {}: {is_authorized}",
        owner_id,
        current_user_id
    );
    Json(AuthorizationResponse {
        authorized: is_authorized,
    })
}
