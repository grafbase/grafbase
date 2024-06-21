use std::{collections::BTreeMap, sync::Arc};

use ascii::AsciiString;
use engine_v2::{Engine, EngineEnv};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use runtime::{cache::GlobalCacheConfig, user_hooks::UserHooks};
use runtime_local::{ComponentLoader, InMemoryCache, InMemoryKvStore, UserHooksWasi, WasiConfig};
use runtime_noop::{trusted_documents::NoopTrustedDocuments, user_hooks::UserHooksNoop};
use tokio::sync::watch;

use crate::config::{AuthenticationConfig, HeaderValue, OperationLimitsConfig, SubgraphConfig, TrustedDocumentsConfig};

/// Send half of the gateway watch channel
#[cfg(not(feature = "lambda"))]
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Engine>>>;

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
    pub trusted_documents: TrustedDocumentsConfig,
    pub wasi: Option<WasiConfig>,
}

/// Creates a new gateway from federated schema.
pub(super) fn generate(
    federated_schema: &str,
    branch_id: Option<ulid::Ulid>,
    config: GatewayConfig,
) -> crate::Result<Engine> {
    let GatewayConfig {
        enable_introspection,
        operation_limits,
        authentication,
        default_headers,
        subgraphs,
        trusted_documents,
        wasi,
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
                development_url: None,
            };

            (name, config)
        })
        .collect();

    let config = engine_config_builder::build_config(&graph_config, graph);

    let cache = InMemoryCache::runtime(GlobalCacheConfig {
        common_cache_tags: Vec::new(),
        enabled: true,
        subdomain: "localhost".to_string(),
    });

    // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
    let trusted_documents = if trusted_documents.enabled {
        let Some(branch_id) = branch_id else {
            return Err(crate::Error::InternalError(
                "Trusted documents are not implemented yet in airgapped mode".into(),
            ));
        };

        runtime::trusted_documents_client::Client::new(super::trusted_documents_client::TrustedDocumentsClient {
            http_client: Default::default(),
            bypass_header: trusted_documents
                .bypass_header
                .bypass_header_name
                .zip(trusted_documents.bypass_header.bypass_header_value)
                .map(|(name, value)| (name.clone().into(), String::from(value.as_ref()))),
            branch_id,
        })
    } else {
        runtime::trusted_documents_client::Client::new(NoopTrustedDocuments)
    };

    let user_hooks = match wasi {
        Some(config) => ComponentLoader::new(config)
            .map_err(|e| crate::Error::InternalError(e.to_string()))?
            .map(UserHooksWasi::new)
            .map(UserHooks::new)
            .unwrap_or_else(|| UserHooks::new(UserHooksNoop)),
        None => UserHooks::new(UserHooksNoop),
    };

    let engine_env = EngineEnv {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        cache: cache.clone(),
        kv: InMemoryKvStore::runtime(),
        trusted_documents,
        meter: grafbase_tracing::metrics::meter_from_global_provider(),
        user_hooks,
    };

    let config = config
        .into_latest()
        .try_into()
        .map_err(|err| crate::Error::InternalError(format!("Failed to generate engine Schema: {err}")))?;

    Ok(Engine::new(Arc::new(config), engine_env))
}
