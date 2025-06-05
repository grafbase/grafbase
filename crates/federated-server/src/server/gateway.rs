mod create_extension_catalog;
mod gateway_runtime;

pub use self::{create_extension_catalog::Error as CreateExtensionCatalogError, gateway_runtime::GatewayRuntime};

use self::create_extension_catalog::create_extension_catalog;
use super::{AccessToken, ObjectStorageResponse};
use engine::Engine;
use gateway_config::Config;
use runtime::trusted_documents_client::{Client, TrustedDocumentsEnforcementMode};
use runtime_local::wasi::hooks::{AccessLogSender, HooksWasi};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::watch;
use ulid::Ulid;

/// Send half of the gateway watch channel
pub(crate) type EngineSender = watch::Sender<Arc<Engine<GatewayRuntime>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

#[derive(Clone)]
pub(crate) enum GraphDefinition {
    /// Response from object storage.
    ObjectStorage {
        response: ObjectStorageResponse,
        object_storage_base_url: url::Url,
    },
    /// Response from static file.
    Sdl(Option<PathBuf>, String),
}

struct Graph {
    federated_sdl: String,
    version_id: Option<Ulid>,
    trusted_documents: Option<Client>,
}

/// Generates a new gateway from the provided graph definition.
///
/// This function takes a `GraphDefinition`, which can be either a response from object storage or a static SDL string,
/// and constructs an `Engine<GatewayRuntime>` based on the provided gateway configuration and optional hot reload settings.
///
/// # Arguments
///
/// - `graph_definition`: The definition of the graph, either from object storage or a static SDL string.
/// - `gateway_config`: The configuration settings for the gateway.
/// - `hot_reload_config_path`: An optional path for hot reload configuration.
/// - `hooks`: The hooks to be used in the gateway.
pub(super) async fn generate(
    graph_definition: GraphDefinition,
    gateway_config: &Config,
    hot_reload_config_path: Option<PathBuf>,
    hooks: HooksWasi,
    access_log: AccessLogSender,
    access_token: Option<&AccessToken>,
) -> crate::Result<Engine<GatewayRuntime>> {
    let (
        current_dir,
        Graph {
            federated_sdl,
            version_id,
            trusted_documents,
        },
    ) = match graph_definition {
        GraphDefinition::ObjectStorage {
            response: object_storage_response,
            object_storage_base_url,
        } => (
            None,
            graph_from_object_storage(
                gateway_config,
                object_storage_response,
                object_storage_base_url,
                access_token,
            ),
        ),
        GraphDefinition::Sdl(current_dir, federated_sdl) => (current_dir, sdl_graph(federated_sdl)),
    };

    tracing::debug!("Creating extension catalog.");
    let extension_catalog = create_extension_catalog(gateway_config).await?;

    tracing::debug!("Building engine Schema.");
    let schema = Arc::new(
        engine::Schema::builder(&federated_sdl)
            .config(gateway_config)
            .extensions(current_dir.as_deref(), &extension_catalog)
            .build()
            .await
            .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?,
    );

    let mut runtime = GatewayRuntime::build(
        gateway_config,
        &extension_catalog,
        &schema,
        hot_reload_config_path,
        version_id,
        hooks,
        access_log,
    )
    .await?;

    if let Some(trusted_documents) = trusted_documents {
        runtime.trusted_documents = trusted_documents;
    }

    Ok(Engine::new(schema, runtime).await)
}

fn sdl_graph(federated_sdl: String) -> Graph {
    Graph {
        federated_sdl,
        version_id: None,
        // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
        trusted_documents: None,
    }
}

fn graph_from_object_storage(
    gateway_config: &Config,
    ObjectStorageResponse {
        branch_id,
        sdl,
        version_id,
        ..
    }: ObjectStorageResponse,
    object_storage_url: url::Url,
    access_token: Option<&AccessToken>,
) -> Graph {
    let trusted_documents = if let (Some(access_token), true) = (access_token, gateway_config.trusted_documents.enabled)
    {
        let enforcement_mode = if gateway_config.trusted_documents.enforced {
            TrustedDocumentsEnforcementMode::Enforce
        } else {
            TrustedDocumentsEnforcementMode::Allow
        };

        Some(runtime::trusted_documents_client::Client::new(
            super::trusted_documents_client::TrustedDocumentsClient::new(
                branch_id,
                gateway_config
                    .trusted_documents
                    .bypass_header
                    .bypass_header_name
                    .as_ref()
                    .zip(
                        gateway_config
                            .trusted_documents
                            .bypass_header
                            .bypass_header_value
                            .as_ref(),
                    )
                    .map(|(name, value)| (name.clone().into(), String::from(value.as_str()))),
                enforcement_mode,
                object_storage_url,
                access_token,
            ),
        ))
    } else {
        None
    };

    Graph {
        federated_sdl: sdl,
        version_id: Some(version_id),
        trusted_documents,
    }
}
