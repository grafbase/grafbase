pub mod create_extension_catalog;
mod gateway_runtime;
mod graph;

pub use self::{create_extension_catalog::Error as CreateExtensionCatalogError, gateway_runtime::GatewayRuntime};
pub use graph::SchemaSource;

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
    schema_source: SchemaSource,
) -> crate::Result<Engine<GatewayRuntime>> {
    // let graph = graph_definition.into_graph(context.gateway_config, context.access_token);

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
        engine::Schema::builder(schema_source.sdl())
            .config(context.gateway_config)
            .extensions(schema_source.parent_dir_path(), &extension_catalog)
            .build()
            .await
            .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?,
    );

    let config = GatewayRuntimeConfig {
        gateway_config: context.gateway_config,
        extension_catalog: &extension_catalog,
        schema: &schema,
        hot_reload_config_path: context.hot_reload_config_path.map(|p| p.to_path_buf()),
        version_id: schema_source.version_id(),
        logging_filter: context.logging_filter.to_string(),
    };

    let mut runtime = GatewayRuntime::build(config).await?;

    if let Some(branch_id) = schema_source.branch_id {
        let enforcement_mode = if config.trusted_documents.enforced {
            TrustedDocumentsEnforcementMode::Enforce
        } else {
            TrustedDocumentsEnforcementMode::Allow
        };

        let bypass_header = config
            .trusted_documents
            .bypass_header
            .bypass_header_name
            .as_ref()
            .zip(config.trusted_documents.bypass_header.bypass_header_value.as_ref())
            .map(|(name, value)| (name.clone().into(), String::from(value.as_str())));

        runtime.trusted_documents = Client::new(
            TrustedDocumentsClient::new(TrustedDocumentsClientConfig {
            branch_id: .branch_id,
            bypass_header,
            enforcement_mode,
            object_storage_url: object_storage_base_url,
            access_token,
        })
        );
    }

    Ok(Engine::new(schema, runtime).await)
}
