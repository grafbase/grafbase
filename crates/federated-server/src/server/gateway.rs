pub mod create_extension_catalog;
mod gateway_runtime;
mod graph;

pub use self::{create_extension_catalog::Error as CreateExtensionCatalogError, gateway_runtime::GatewayRuntime};
pub use graph::GraphDefinition;

use self::create_extension_catalog::create_extension_catalog;
use super::AccessToken;
use engine::Engine;
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;
use gateway_runtime::GatewayRuntimeConfig;
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
pub(crate) type EngineSender = watch::Sender<Arc<Engine<GatewayRuntime>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

/// Generates a new gateway from the provided graph definition.
pub(super) async fn generate(
    context: EngineBuildContext<'_>,
    graph_definition: GraphDefinition,
) -> crate::Result<Engine<GatewayRuntime>> {
    let graph = graph_definition.into_graph(context.gateway_config, context.access_token);

    let extension_catalog = match context.extension_catalog {
        Some(catalog) => Cow::Borrowed(catalog),
        None => {
            tracing::debug!("Creating extension catalog.");
            let (catalog, _) = create_extension_catalog(context.gateway_config).await?;

            Cow::Owned(catalog)
        }
    };

    tracing::debug!("Building engine Schema.");

    let schema = Arc::new(
        engine::Schema::builder(&graph.federated_sdl)
            .config(context.gateway_config)
            .extensions(graph.current_dir.as_deref(), &extension_catalog)
            .build()
            .await
            .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?,
    );

    let config = GatewayRuntimeConfig {
        gateway_config: context.gateway_config,
        extension_catalog: &extension_catalog,
        schema: &schema,
        hot_reload_config_path: context.hot_reload_config_path.map(|p| p.to_path_buf()),
        version_id: graph.version_id,
        logging_filter: context.logging_filter.to_string(),
    };

    let mut runtime = GatewayRuntime::build(config).await?;

    if let Some(trusted_documents) = graph.trusted_documents {
        runtime.trusted_documents = trusted_documents;
    }

    Ok(Engine::new(schema, runtime).await)
}
