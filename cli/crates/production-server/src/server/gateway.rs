use std::sync::Arc;

use engine_v2::EngineEnv;
use gateway_v2::{Gateway, GatewayEnv};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::FederatedGraphConfig;
use runtime::{cache::GlobalCacheConfig, trusted_documents_service::TrustedDocumentsClient};
use runtime_local::{InMemoryCache, InMemoryKvStore};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

use crate::config::{AuthenticationConfig, OperationLimitsConfig};

/// Send half of the gateway watch channel
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Gateway>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type GatewayWatcher = watch::Receiver<Option<Arc<Gateway>>>;

/// Creates a new gateway from federated schema.
pub(super) fn generate(
    federated_schema: &str,
    operation_limits: Option<OperationLimitsConfig>,
    authentication: Option<AuthenticationConfig>,
    enable_introspection: bool,
) -> crate::Result<Gateway> {
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

    let config = engine_config_builder::build_config(&graph_config, graph);

    let cache = InMemoryCache::runtime(GlobalCacheConfig {
        common_cache_tags: Vec::new(),
        enabled: true,
        subdomain: "localhost".to_string(),
    });

    let engine_env = EngineEnv {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        cache: cache.clone(),
        // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
        // TODO: https://linear.app/grafbase/issue/GB-6169/support-trusted-documents-in-hybrid-mode
        trusted_documents: TrustedDocumentsClient::new(Box::new(NoopTrustedDocuments), String::new()),
    };

    let gateway_env = GatewayEnv {
        kv: InMemoryKvStore::runtime(),
        cache,
    };

    Ok(Gateway::new(config.into_latest().into(), engine_env, gateway_env))
}
