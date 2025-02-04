use crate::Error;

use super::GdnResponse;
use engine::{Engine, SchemaVersion};
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Manifest, VersionedManifest};
use gateway_config::{Config, WasiExtensionsConfig};
use graphql_composition::FederatedGraph;
use runtime::trusted_documents_client::{Client, TrustedDocumentsEnforcementMode};
use runtime_local::wasi::{
    extensions::{Directive, ExtensionConfig, ExtensionType, WasiExtensions},
    hooks::{ChannelLogSender, HooksWasi},
};
use std::{env, fs::File, ops::Not, path::PathBuf, sync::Arc};
use tokio::sync::watch;
use ulid::Ulid;

pub use gateway_runtime::GatewayRuntime;

mod gateway_runtime;

/// Send half of the gateway watch channel
pub(crate) type EngineSender = watch::Sender<Arc<Engine<GatewayRuntime>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Arc<Engine<GatewayRuntime>>>;

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
    access_log: ChannelLogSender,
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

    let extension_catalog = create_extension_catalog(gateway_config)?;

    let federated_graph =
        FederatedGraph::from_sdl(&federated_sdl).map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;

    let schema = engine::Schema::build(gateway_config, federated_graph, &extension_catalog, schema_version)
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
        runtime.extensions = WasiExtensions::new(access_log, extensions)
            .await
            .map_err(|e| Error::InternalError(e.to_string()))?;
    }

    Ok(Engine::new(Arc::new(schema), runtime).await)
}

fn create_wasi_extension_configs(
    extension_catalog: &ExtensionCatalog,
    gateway_config: &Config,
    schema: &engine::Schema,
) -> Option<Vec<ExtensionConfig>> {
    let mut wasi_extensions: Vec<ExtensionConfig> = Vec::with_capacity(extension_catalog.len());

    let extension_configs = gateway_config.extensions.as_ref()?;

    for (id, extension) in extension_catalog.iter().enumerate() {
        let extension_config = extension_configs
            .get(extension.manifest.name())
            .expect("we made sure in the create_extension_catalog that this extension is in the config");

        let extension_type = match &extension.manifest.kind {
            extension_catalog::Kind::FieldResolver(_) => ExtensionType::Resolver,
        };

        let wasi_config = WasiExtensionsConfig {
            location: extension.wasm_path.clone(),
            networking: extension_config.networking(),
            stdout: extension_config.stdout(),
            stderr: extension_config.stderr(),
            environment_variables: extension_config.environment_variables(),
        };

        wasi_extensions.push(ExtensionConfig {
            id: ExtensionId::from(id),
            name: extension.manifest.name().to_owned(),
            version: extension.manifest.version().to_owned(),
            extension_type,
            schema_directives: Vec::new(),
            max_pool_size: extension_config.max_pool_size(),
            wasi_config,
            // TODO: we actually need to pass the extension config here, sigh :(
            extension_config: Vec::new(),
        });
    }

    for subgraph in schema.subgraphs() {
        let directives = subgraph.extension_schema_directives();

        for schema_directive in directives {
            let config = &mut wasi_extensions[usize::from(schema_directive.extension_id)];

            let directive = match schema_directive.arguments() {
                Some(args) => Directive::new(config.name.clone(), subgraph.name().to_string(), &args),
                None => Directive::new(config.name.clone(), subgraph.name().to_string(), &Option::<()>::None),
            };

            config.schema_directives.push(directive);
        }
    }

    wasi_extensions.is_empty().not().then_some(wasi_extensions)
}

// TODO: with lock file this will be smarter...
fn create_extension_catalog(gateway_config: &Config) -> crate::Result<ExtensionCatalog> {
    let mut catalog = ExtensionCatalog::default();

    let Some(ref extension_configs) = gateway_config.extensions else {
        return Ok(catalog);
    };

    for (_, config) in extension_configs.iter() {
        let Some(path) = config.path() else {
            continue;
        };

        let Ok(mut extension_dir) = path.read_dir() else {
            continue;
        };

        if !extension_dir.all(|entry| {
            entry
                .map(|e| e.file_name() == "extension.wasm" || e.file_name() == "manifest.json")
                .unwrap_or(false)
        }) {
            continue;
        }

        let manifest_data = File::open(path.join("manifest.json")).map_err(|e| Error::InternalError(e.to_string()))?;

        let manifest: VersionedManifest =
            serde_json::from_reader(manifest_data).map_err(|e| Error::InternalError(e.to_string()))?;
        let manifest = manifest.into_latest();

        let extension = Extension {
            manifest,
            wasm_path: path.join("extension.wasm").canonicalize().unwrap(),
        };

        catalog.push(extension);
    }

    let Ok(grafbase_extensions) = env::current_dir()
        .map_err(|e| Error::InternalError(e.to_string()))?
        .join("grafbase_extensions")
        .read_dir()
    else {
        return Ok(catalog);
    };

    for extension_dir in grafbase_extensions {
        let extension_dir = extension_dir.map_err(|e| Error::InternalError(e.to_string()))?;

        if !extension_dir.path().is_dir() {
            continue;
        }

        let extension_dir = extension_dir
            .path()
            .read_dir()
            .map_err(|e| Error::InternalError(e.to_string()))?;

        let mut manifest = None;
        let mut wasm_path = None;

        for file in extension_dir {
            let file = file.map_err(|e| Error::InternalError(e.to_string()))?;

            if file.path().is_dir() {
                continue;
            }

            let path = file.path();
            let file_name = path.file_name().and_then(|n| n.to_str());

            if file_name == Some("manifest.json") {
                let manifest_data = File::open(file.path()).map_err(|e| Error::InternalError(e.to_string()))?;

                let manifest_data: Manifest =
                    serde_json::from_reader(manifest_data).map_err(|e| Error::InternalError(e.to_string()))?;

                manifest = Some(manifest_data);

                continue;
            }

            if file_name == Some("extension.wasm") {
                wasm_path = Some(file.path().to_path_buf());
            }
        }

        if let Some((wasm_path, manifest)) = wasm_path.zip(manifest) {
            if extension_configs
                .get(manifest.name())
                .filter(|c| c.version().matches(manifest.version()))
                .is_none()
            {
                continue;
            }

            let extension = Extension {
                manifest,
                wasm_path: wasm_path.canonicalize().unwrap(),
            };

            catalog.push(extension);
        }
    }

    Ok(catalog)
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
