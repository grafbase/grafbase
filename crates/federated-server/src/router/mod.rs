mod cors;
mod csrf;
mod graphql;
mod health;
pub(crate) mod layers;
mod state;

use std::pin::Pin;

use axum::{body::Bytes, routing::get};
use runtime::{authentication::Authenticate, extension::HooksExtension};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tower_http::{
    compression::{CompressionLayer, DefaultPredicate, Predicate, predicate::NotForContentType},
    cors::CorsLayer,
};

use crate::{engine::EngineWatcher, router::state::ServerState};

use super::ServerRuntime;

pub struct RouterConfig<R, SR, H, F>
where
    R: engine::Runtime,
    SR: ServerRuntime,
    H: HooksExtension,
    F: FnOnce(axum::Router<()>) -> axum::Router<()>,
{
    pub config: gateway_config::Config,
    pub engine: EngineWatcher<R>,
    pub server_runtime: SR,
    pub extensions: H,
    pub inject_layers_before_cors: F,
}

pub async fn create<R, SR, H, F>(
    RouterConfig {
        config,
        engine,
        server_runtime,
        extensions,
        inject_layers_before_cors,
    }: RouterConfig<R, SR, H, F>,
) -> crate::Result<(axum::Router, Option<CancellationToken>)>
where
    R: engine::Runtime,
    SR: ServerRuntime,
    H: HooksExtension,
    F: FnOnce(axum::Router<()>) -> axum::Router<()>,
{
    let path = &config.graph.path;
    let websocket_path = &config.graph.websocket_path;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = graphql::ws::WebsocketAccepter::new(websocket_receiver, engine.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(ref cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let state = ServerState::new(
        engine.clone(),
        config.request_body_limit.bytes().max(0) as usize,
        server_runtime.clone(),
    );

    let mut router = server_runtime
        .base_router()
        .unwrap_or_default()
        .route(path, get(graphql::http::execute).post(graphql::http::execute))
        .route_service(websocket_path, graphql::ws::WebsocketService::new(websocket_sender))
        .with_state(state);

    let ct = match &config.mcp {
        Some(mcp_config) if mcp_config.enabled => {
            let (mcp_router, ct) = grafbase_mcp::router(&engine, mcp_config);
            router = router.merge(mcp_router);
            ct
        }
        _ => None,
    };

    for public_metadata_endpoint in engine
        .borrow()
        .runtime
        .authentication()
        .public_metadata_endpoints()
        .await
        .unwrap_or_default()
    {
        router = router.route(
            &public_metadata_endpoint.path,
            get(public_metadata_handler(
                public_metadata_endpoint.response_body.into(),
                public_metadata_endpoint.headers,
            )),
        );
    }

    // Streaming and compression doesn't really work well today. Had a panic deep inside stream
    // unfold. Furthermore there seem to be issues with it as pointed out by Apollo's router
    // team:
    // https://github.com/tower-rs/tower-http/issues/292
    // They have copied the compression code and adjusted it, see PRs for:
    // https://github.com/apollographql/router/issues/1572
    // We'll need to see what we do. For now I'm disabling it as it's not important enough
    // right now.
    let compression =
        CompressionLayer::new().compress_when(DefaultPredicate::new().and(
            NotForContentType::const_new("multipart/mixed").and(NotForContentType::const_new("text/event-stream")),
        ));

    let mut router = inject_layers_before_cors(router)
        .layer(layers::HooksLayer::new(extensions))
        .layer(compression)
        .layer(cors);

    if config.csrf.enabled {
        router = csrf::inject_layer(router, &config.csrf);
    }

    if config.health.enabled {
        if let Some(listen) = config.health.listen {
            tokio::spawn(health::bind_health_endpoint(listen, config.tls.clone(), config.health));
        } else {
            router = router.route(&config.health.path, get(health::health));
        }
    }

    Ok((router, ct))
}

/// Creates a handler for public metadata endpoints that returns a pre-configured response.
fn public_metadata_handler(
    response_body: Bytes,
    headers: http::HeaderMap,
) -> impl FnOnce() -> Pin<Box<dyn Future<Output = axum::response::Response> + Send + Sync + 'static>> + Clone {
    move || {
        let headers = headers.clone();
        let response_body = response_body.clone();
        Box::pin(async move {
            let mut response = axum::response::Response::new(axum::body::Body::from(response_body));

            *response.headers_mut() = headers;

            response
        })
    }
}
