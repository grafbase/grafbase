mod graphql;
mod health;
pub(crate) mod layers;
mod public_metadata;
mod state;

use std::{net::SocketAddr, sync::Arc};

use axum::routing::get;
use engine::ContractAwareEngine;
use extension_catalog::ExtensionCatalog;
use gateway_config::{AuthenticationResourcesConfig, Config};
use runtime::extension::GatewayExtensions;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tower_http::{
    compression::{CompressionLayer, DefaultPredicate, Predicate, predicate::NotForContentType},
    cors::CorsLayer,
};

use crate::router::{
    layers::{ExtensionLayer, TelemetryLayer},
    state::ServerState,
};

use super::ServerRuntime;

pub struct RouterConfig<R, SR, E>
where
    R: engine::Runtime,
    SR: ServerRuntime,
    E: GatewayExtensions,
{
    pub config: Config,
    pub extension_catalog: ExtensionCatalog,
    pub engine: EngineWatcher<R>,
    pub server_runtime: SR,
    pub extensions: E,
    pub listen_address: Option<SocketAddr>,
}

pub type EngineWatcher<R> = watch::Receiver<Arc<ContractAwareEngine<R>>>;

pub async fn create<R, SR, E>(
    RouterConfig {
        config,
        extension_catalog,
        engine,
        server_runtime,
        extensions,
        listen_address,
    }: RouterConfig<R, SR, E>,
) -> crate::Result<(axum::Router, Option<CancellationToken>)>
where
    R: engine::Runtime,
    SR: ServerRuntime,
    E: GatewayExtensions,
{
    let telemetry = TelemetryLayer::new_from_global_meter_provider(listen_address);
    let common_layers = {
        let cors = match config.cors {
            Some(ref cors_config) => layers::cors_layer(cors_config),
            None => CorsLayer::permissive(),
        };
        let csrf = layers::CsrfLayer::new(&config.csrf);

        // Streaming and compression doesn't really work well today. Had a panic deep inside stream
        // unfold. Furthermore there seem to be issues with it as pointed out by Apollo's router
        // team:
        // https://github.com/tower-rs/tower-http/issues/292
        // They have copied the compression code and adjusted it, see PRs for:
        // https://github.com/apollographql/router/issues/1572
        // We'll need to see what we do. For now I'm disabling it as it's not important enough
        // right now.
        let compression = CompressionLayer::new().compress_when(DefaultPredicate::new().and(
            NotForContentType::const_new("multipart/mixed").and(NotForContentType::const_new("text/event-stream")),
        ));

        tower::ServiceBuilder::new().layer(cors).layer(csrf).layer(compression)
    };

    let mut router = server_runtime
        .base_router()
        .unwrap_or_default()
        .fallback(fallback)
        .layer(common_layers.clone().layer(telemetry.clone()));

    // Protected routes that need authentication
    let graphql = axum::Router::new()
        //
        // == /graphql ==
        //
        .route(
            &config.graph.path,
            get(graphql::http::execute).post(graphql::http::execute),
        )
        //
        // == /ws ==
        //
        .route_service(&config.graph.websocket_path, {
            let (websocket_sender, websocket_receiver) = mpsc::channel(16);
            let websocket_accepter = graphql::ws::WebsocketAccepter::new(websocket_receiver, engine.clone());

            tokio::spawn(websocket_accepter.handler());
            graphql::ws::WebsocketService::new(websocket_sender)
        })
        //
        // State
        //
        .with_state(ServerState::new(
            engine.clone(),
            config.request_body_limit.bytes().max(0) as usize,
            server_runtime.clone(),
        ))
        //
        // Layers
        //
        .layer(
            common_layers
                .clone()
                .layer(telemetry.clone().with_route(&config.graph.path))
                .layer(build_extension_layer(
                    &config,
                    &extension_catalog,
                    &extensions,
                    &config.authentication.protected_resources.graphql,
                )?),
        );

    router = router.merge(graphql);

    //
    // Public metadata endpoints
    //
    let public_metadata_endpoints = extensions.public_metadata_endpoints().await?;
    if !public_metadata_endpoints.is_empty() {
        let mut public_router = axum::Router::new();
        for endpoint in public_metadata_endpoints {
            public_router = public_router.route(
                &endpoint.path,
                get(public_metadata::handler(
                    endpoint.response_body.into(),
                    endpoint.headers,
                )),
            );
        }
        router = router.merge(public_router.layer(telemetry.clone()).layer(common_layers.clone()));
    }

    //
    // == /mcp ==
    //
    let ct = match &config.mcp {
        Some(mcp_config) if mcp_config.enabled => {
            let (mcp_router, ct) = grafbase_mcp::router(&engine, mcp_config);
            router = router.merge(
                mcp_router.layer(
                    common_layers
                        .clone()
                        .layer(telemetry.clone().with_route(&mcp_config.path))
                        .layer(build_extension_layer(
                            &config,
                            &extension_catalog,
                            &extensions,
                            &config.authentication.protected_resources.mcp,
                        )?),
                ),
            );
            ct
        }
        _ => None,
    };

    //
    // == /health ==
    //
    if config.health.enabled {
        if let Some(listen) = config.health.listen {
            tokio::spawn(health::bind_health_endpoint(listen, config.tls.clone(), config.health));
        } else {
            router = router.route(&config.health.path, get(health::health));
        }
    }

    Ok((router, ct))
}

fn build_extension_layer<E: GatewayExtensions>(
    gateway_config: &Config,
    extension_catalog: &ExtensionCatalog,
    extensions: &E,
    config: &AuthenticationResourcesConfig,
) -> crate::Result<ExtensionLayer<E>> {
    let extension_ids = config
        .extensions
        .as_ref()
        .map(|keys| {
            keys.iter()
                .map(|key| {
                    let ext = extension_catalog
                        .get_id_by_config_key(key)
                        .ok_or_else(|| format!("Could not find extension named {key}"))?;
                    if !extension_catalog[ext].manifest.is_authentication() {
                        Err(format!("Extension {key} is not an authentication extension"))
                    } else {
                        Ok(ext)
                    }
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_else(|| {
            extension_catalog
                .iter_with_id()
                .filter_map(|(id, ext)| {
                    if ext.manifest.is_authentication() {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        });
    Ok(layers::ExtensionLayer::new(
        extensions.clone(),
        gateway_config.graph.contracts.default_key.clone(),
        extension_ids,
        config.default.or(gateway_config.authentication.default),
    ))
}

async fn fallback() -> (http::StatusCode, &'static str) {
    (http::StatusCode::NOT_FOUND, "Not Found")
}
