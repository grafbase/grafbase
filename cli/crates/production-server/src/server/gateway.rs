use std::{collections::BTreeMap, sync::Arc};

use ascii::AsciiString;
use engine_v2::{Engine, EngineEnv};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use runtime::trusted_documents_service::TrustedDocumentsClient;
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

use crate::config::{AuthenticationConfig, HeaderValue, OperationLimitsConfig, SubgraphConfig};

/// Send half of the gateway watch channel
pub(crate) type EngineSender = watch::Sender<Option<Arc<Engine>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Option<Arc<Engine>>>;

#[derive(Debug, Clone)]
pub(crate) struct GatewayConfig {
    pub enable_introspection: bool,
    pub operation_limits: Option<OperationLimitsConfig>,
    pub authentication: Option<AuthenticationConfig>,
    pub default_headers: BTreeMap<AsciiString, HeaderValue>,
    pub subgraphs: BTreeMap<String, SubgraphConfig>,
}

/// Creates a new gateway from federated schema.
pub(super) fn generate(federated_schema: &str, config: GatewayConfig) -> crate::Result<Gateway> {
    let GatewayConfig {
        enable_introspection,
        operation_limits,
        authentication,
        default_headers,
        subgraphs,
    } = config;

    let graph =
        FederatedGraph::from_sdl(federated_schema).map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;

    let mut graph_config = FederatedGraphConfig::default();

    if let Some(limits_config) = operation_limits {
        graph_config.operation_limits = limits_config.into();
    }

    if let Some(auth_config) = authentication {
        graph_config.auth = Some(auth_config.into());
    }

    graph_config.disable_introspection = !enable_introspection;

    graph_config.default_headers = default_headers
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.into()))
        .collect();

    graph_config.subgraphs = subgraphs
        .into_iter()
        .map(|(name, value)| {
            let headers = value
                .headers
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.into()))
                .collect();

            let config = parser_sdl::federation::SubgraphConfig {
                name: name.clone(),
                websocket_url: value.websocket_url.map(|url| url.to_string()),
                headers,
            };

            (name, config)
        })
        .collect();

    let config = engine_config_builder::build_config(&graph_config, graph);
    let async_runtime = runtime_local::TokioCurrentRuntime::runtime();
    let cache = runtime_local::InMemoryCacheV2::runtime(async_runtime.clone());
    Ok(Engine::new(
        config.into_latest().into(),
        ulid::Ulid::new().to_string().into(),
        EngineEnv {
            async_runtime,
            cache,
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
            // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
            // TODO: https://linear.app/grafbase/issue/GB-6169/support-trusted-documents-in-hybrid-mode
            trusted_documents: TrustedDocumentsClient::new(Box::new(NoopTrustedDocuments), String::new()),
            cache_opeartion_cache_control: false,
            kv: runtime_local::InMemoryKvStore::runtime(),
        },
    ))
}
