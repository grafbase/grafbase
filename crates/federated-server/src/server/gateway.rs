mod create_extension_catalog;
mod gateway_runtime;

pub use self::{create_extension_catalog::Error as CreateExtensionCatalogError, gateway_runtime::GatewayRuntime};

use self::create_extension_catalog::create_extension_catalog;
use super::GdnResponse;
use crate::Error;
use engine::{Engine, SchemaVersion};
use extension_catalog::{ExtensionCatalog, ExtensionId, KindDiscriminants};
use gateway_config::{AuthenticationProvider, Config, WasiExtensionsConfig};
use graphql_composition::FederatedGraph;
use runtime::{
    extension::AuthorizerId,
    trusted_documents_client::{Client, TrustedDocumentsEnforcementMode},
};
use runtime_local::wasi::hooks::{AccessLogSender, HooksWasi};
use std::{ops::Not, path::PathBuf, sync::Arc};
use tokio::sync::watch;
use ulid::Ulid;
use wasi_component_loader::{
    extension::{ExtensionConfig, ExtensionGuestConfig, ExtensionPoolId, ExtensionsWasiRuntime, SchemaDirective},
    resources::SharedResources,
};

/// Send half of the gateway watch channel
pub(crate) type EngineSender = watch::Sender<Arc<Engine<GatewayRuntime>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher<R> = watch::Receiver<Arc<Engine<R>>>;

#[derive(Clone)]
pub(crate) enum GraphDefinition {
    /// Response from GDN.
    Gdn(GdnResponse),
    /// Response from static file.
    Sdl(String),
}

struct Graph {
    federated_sdl: String,
    schema_version: SchemaVersion,
    version_id: Option<Ulid>,
    trusted_documents: Option<Client>,
}

/// Generates a new gateway from the provided graph definition.
///
/// This function takes a `GraphDefinition`, which can be either a response from GDN or a static SDL string,
/// and constructs an `Engine<GatewayRuntime>` based on the provided gateway configuration and optional hot reload settings.
///
/// # Arguments
///
/// - `graph_definition`: The definition of the graph, either from GDN or a static SDL string.
/// - `gateway_config`: The configuration settings for the gateway.
/// - `hot_reload_config_path`: An optional path for hot reload configuration.
/// - `hooks`: The hooks to be used in the gateway.
pub(super) async fn generate(
    graph_definition: GraphDefinition,
    gateway_config: &Config,
    hot_reload_config_path: Option<PathBuf>,
    hooks: HooksWasi,
    access_log: AccessLogSender,
) -> crate::Result<Engine<GatewayRuntime>> {
    let Graph {
        federated_sdl,
        schema_version,
        version_id,
        trusted_documents,
    } = match graph_definition {
        GraphDefinition::Gdn(gdn_response) => gdn_graph(gateway_config, gdn_response),
        GraphDefinition::Sdl(federated_sdl) => sdl_graph(federated_sdl),
    };

    let extension_catalog = create_extension_catalog(gateway_config).await?;

    let federated_graph =
        FederatedGraph::from_sdl(&federated_sdl).map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;

    let schema = engine::Schema::build(gateway_config, &federated_graph, &extension_catalog, schema_version)
        .await
        .map_err(|err| crate::Error::SchemaValidationError(err.to_string()))?;

    let mut runtime = GatewayRuntime::build(
        gateway_config,
        hot_reload_config_path,
        version_id,
        hooks,
        Default::default(),
    )
    .await?;

    if let Some(trusted_documents) = trusted_documents {
        runtime.trusted_documents = trusted_documents;
    }

    if let Some(extensions) = create_wasi_extension_configs(&extension_catalog, gateway_config, &schema) {
        runtime.extensions = ExtensionsWasiRuntime::new(SharedResources { access_log }, extensions)
            .await
            .map_err(|e| Error::InternalError(e.to_string()))?;
    }

    Ok(Engine::new(Arc::new(schema), runtime).await)
}

fn create_wasi_extension_configs(
    extension_catalog: &ExtensionCatalog,
    gateway_config: &Config,
    schema: &engine::Schema,
) -> Option<Vec<ExtensionConfig<Option<toml::Value>>>> {
    let mut wasi_extensions = Vec::with_capacity(extension_catalog.len());

    let extension_configs = gateway_config.extensions.as_ref()?;

    for (id, extension) in extension_catalog.iter().enumerate() {
        let extension_config = extension_configs
            .get(extension.manifest.name())
            .expect("we made sure in the create_extension_catalog that this extension is in the config");

        let wasi_config = WasiExtensionsConfig {
            location: extension.wasm_path.clone(),
            networking: extension_config
                .networking()
                .unwrap_or_else(|| extension.manifest.network_enabled()),
            stdout: extension_config
                .stdout()
                .unwrap_or_else(|| extension.manifest.stdout_enabled()),
            stderr: extension_config
                .stderr()
                .unwrap_or_else(|| extension.manifest.stderr_enabled()),
            environment_variables: extension_config
                .environment_variables()
                .unwrap_or_else(|| extension.manifest.environment_variables_enabled()),
        };

        let max_pool_size = extension_config.max_pool_size();
        let id = ExtensionId::from(id);

        let r#type = KindDiscriminants::from(&extension.manifest.kind);
        match r#type {
            KindDiscriminants::FieldResolver => {
                let id = ExtensionPoolId::Resolver(id);

                wasi_extensions.push(ExtensionConfig {
                    id,
                    manifest_id: extension.manifest.id.clone(),
                    max_pool_size,
                    wasi_config,
                    guest_config: ExtensionGuestConfig {
                        r#type,
                        schema_directives: Vec::new(),
                        configuration: extension_config.config().cloned(),
                    },
                });
            }
            KindDiscriminants::Authenticator => {
                let Some(auth_config) = gateway_config.authentication.as_ref() else {
                    continue;
                };

                for (auth_id, provider) in auth_config.providers.iter().enumerate() {
                    let AuthenticationProvider::Extension(extension_provider) = provider else {
                        continue;
                    };

                    if extension_provider.extension != extension.manifest.name() {
                        continue;
                    }

                    let id = ExtensionPoolId::Authorizer(id, AuthorizerId::from(auth_id));

                    wasi_extensions.push(ExtensionConfig {
                        id,
                        manifest_id: extension.manifest.id.clone(),
                        max_pool_size,
                        wasi_config: wasi_config.clone(),
                        guest_config: ExtensionGuestConfig {
                            r#type,
                            schema_directives: Vec::new(),
                            configuration: extension_provider.config.clone(),
                        },
                    });
                }
            }
        }
    }

    for subgraph in schema.subgraphs() {
        let directives = subgraph.extension_schema_directives();

        for schema_directive in directives {
            let config = &mut wasi_extensions[usize::from(schema_directive.extension_id)];

            config.guest_config.schema_directives.push(SchemaDirective::new(
                schema_directive.name(),
                subgraph.name(),
                schema_directive.static_arguments(),
            ));
        }
    }

    wasi_extensions.is_empty().not().then_some(wasi_extensions)
}

fn sdl_graph(federated_sdl: String) -> Graph {
    let version = engine::SchemaVersion::from(
        [
            b"hash:".to_vec(),
            blake3::hash(federated_sdl.as_bytes()).as_bytes().to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>(),
    );

    Graph {
        federated_sdl,
        schema_version: version,
        version_id: None,
        // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
        trusted_documents: None,
    }
}

fn gdn_graph(
    gateway_config: &Config,
    GdnResponse {
        branch_id,
        sdl,
        version_id,
        ..
    }: GdnResponse,
) -> Graph {
    let version = engine::SchemaVersion::from(
        [b"id:".to_vec(), version_id.to_bytes().to_vec()]
            .into_iter()
            .flatten()
            .collect::<Vec<u8>>(),
    );

    let trusted_documents = if gateway_config.trusted_documents.enabled {
        let enforcement_mode = if gateway_config.trusted_documents.enforced {
            TrustedDocumentsEnforcementMode::Enforce
        } else {
            TrustedDocumentsEnforcementMode::Allow
        };

        Some(runtime::trusted_documents_client::Client::new(
            super::trusted_documents_client::TrustedDocumentsClient::new(
                Default::default(),
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
                    .map(|(name, value)| (name.clone().into(), String::from(value.as_ref()))),
                enforcement_mode,
            ),
        ))
    } else {
        None
    };

    Graph {
        federated_sdl: sdl,
        schema_version: version,
        version_id: Some(version_id),
        trusted_documents,
    }
}
