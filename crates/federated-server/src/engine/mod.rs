mod reloader;
mod runtime;
mod trusted_documents_client;

use crate::{extensions::create_extension_catalog, graph::Graph};

pub use self::runtime::*;
use axum::response::IntoResponse as _;
use engine::{Body, ContractAwareEngine};
pub(crate) use reloader::*;
use wasi_component_loader::extension::GatewayWasmExtensions;

use super::AccessToken;
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;
use std::{path::PathBuf, sync::Arc};

/// Context struct that bundles all the semi-static parameters needed to build an engine.
#[derive(Clone, Copy)]
pub(super) struct EngineBuildContext<'a> {
    pub gateway_config: &'a Config,
    pub hot_reload_config_path: Option<&'a PathBuf>,
    pub access_token: Option<&'a AccessToken>,
    pub extension_catalog: Option<&'a Arc<ExtensionCatalog>>,
    pub logging_filter: &'a str,
    pub gateway_extensions: &'a GatewayWasmExtensions,
}

/// Generates a new gateway from the provided graph definition.
pub(super) async fn generate(
    context: EngineBuildContext<'_>,
    graph: Graph,
) -> crate::Result<ContractAwareEngine<EngineRuntime>> {
    // let graph = graph_definition.into_graph(context.gateway_config, context.access_token);

    let extension_catalog = match context.extension_catalog {
        Some(catalog) => catalog.clone(),
        None => {
            tracing::debug!("Creating extension catalog.");
            Arc::new(create_extension_catalog(context.gateway_config).await?)
        }
    };

    tracing::debug!("Building engine Schema.");

    let schema = Arc::new(
        engine::Schema::builder(graph.sdl())
            .config(Arc::new(context.gateway_config.clone()))
            .extensions(&extension_catalog)
            .build()
            .await
            .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?,
    );

    let runtime = EngineRuntime::build(context, &graph, &schema, extension_catalog).await?;

    Ok(ContractAwareEngine::new(schema, runtime))
}

pub(crate) fn into_axum_response(response: http::Response<Body>) -> axum::response::Response {
    let (parts, body) = response.into_parts();
    match body {
        Body::Bytes(bytes) => (parts.status, parts.headers, parts.extensions, bytes).into_response(),
        Body::Stream(stream) => (
            parts.status,
            parts.headers,
            parts.extensions,
            axum::body::Body::from_stream(stream),
        )
            .into_response(),
    }
}
