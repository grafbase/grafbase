mod reloader;
mod runtime;
mod trusted_documents_client;

use crate::{extensions::create_extension_catalog, graph::Graph};

pub use self::runtime::*;
use axum::response::IntoResponse as _;
use engine::Body;
pub(crate) use reloader::*;

use super::AccessToken;
use ::engine::Engine;
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;
use std::{borrow::Cow, path::PathBuf, sync::Arc};
use tokio::sync::watch;

/// Context struct that bundles all the semi-static parameters needed to build an engine.
#[derive(Clone, Copy)]
pub(super) struct EngineBuildContext<'a> {
    pub gateway_config: &'a Config,
    pub hot_reload_config_path: Option<&'a PathBuf>,
    pub access_token: Option<&'a AccessToken>,
    pub extension_catalog: Option<&'a ExtensionCatalog>,
    pub logging_filter: &'a str,
}

/// Send half of the gateway watch channel
pub(crate) type EngineSender = watch::Sender<Arc<Engine<EngineRuntime>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

/// Generates a new gateway from the provided graph definition.
pub(super) async fn generate(context: EngineBuildContext<'_>, graph: Graph) -> crate::Result<Engine<EngineRuntime>> {
    // let graph = graph_definition.into_graph(context.gateway_config, context.access_token);

    let extension_catalog = match context.extension_catalog {
        Some(catalog) => Cow::Borrowed(catalog),
        None => {
            tracing::debug!("Creating extension catalog.");
            let catalog = create_extension_catalog(context.gateway_config).await?;

            Cow::Owned(catalog)
        }
    };

    tracing::debug!("Building engine Schema.");

    let schema = Arc::new(
        engine::Schema::builder(graph.sdl())
            .config(context.gateway_config)
            .extensions(graph.parent_dir_path(), &extension_catalog)
            .build()
            .await
            .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?,
    );

    let runtime = EngineRuntime::build(context, &graph, &schema, &extension_catalog).await?;

    Ok(Engine::new(schema, runtime).await)
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
