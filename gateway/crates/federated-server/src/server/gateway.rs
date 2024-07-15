use std::{collections::BTreeMap, sync::Arc};

use engine_v2::{Engine, InMemoryRateLimiter};
use graphql_composition::FederatedGraph;
use parser_sdl::federation::{header::SubgraphHeaderRule, FederatedGraphConfig};
use runtime_local::{ComponentLoader, HooksConfig, HooksWasi, InMemoryKvStore};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

use crate::{
    config::{AuthenticationConfig, OperationLimitsConfig, RateLimitConfig, SubgraphConfig, TrustedDocumentsConfig},
    HeaderRule,
};

/// Send half of the gateway watch channel
#[cfg(not(feature = "lambda"))]
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Option<Arc<Engine<GatewayRuntime>>>>;

#[derive(Debug, Clone)]
pub(crate) struct GatewayConfig {
    pub enable_introspection: bool,
    pub operation_limits: Option<OperationLimitsConfig>,
    pub authentication: Option<AuthenticationConfig>,
    pub header_rules: Vec<HeaderRule>,
    pub subgraphs: BTreeMap<String, SubgraphConfig>,
    pub trusted_documents: TrustedDocumentsConfig,
    pub wasi: Option<HooksConfig>,
    pub rate_limit: Option<RateLimitConfig>,
}

/// Creates a new gateway from federated schema.
pub(super) async fn generate(
    federated_schema: &str,
    branch_id: Option<ulid::Ulid>,
    config: GatewayConfig,
) -> crate::Result<Engine<GatewayRuntime>> {
    let GatewayConfig {
        enable_introspection,
        operation_limits,
        authentication,
        header_rules,
        subgraphs,
        trusted_documents,
        wasi,
        rate_limit,
    } = config;

    let schema_version = blake3::hash(federated_schema.as_bytes());
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

    graph_config.header_rules = header_rules.into_iter().map(SubgraphHeaderRule::from).collect();

    graph_config.rate_limit = rate_limit.map(Into::into);

    graph_config.subgraphs = subgraphs
        .into_iter()
        .map(|(name, value)| {
            let header_rules = value.headers.into_iter().map(SubgraphHeaderRule::from).collect();

            let config = parser_sdl::federation::SubgraphConfig {
                name: name.clone(),
                websocket_url: value.websocket_url.map(|url| url.to_string()),
                header_rules,
                development_url: None,
                rate_limit: value.rate_limit.map(Into::into),
            };

            (name, config)
        })
        .collect();

    let config = engine_config_builder::build_config(&graph_config, graph).into_latest();

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

    let rate_limiter = InMemoryRateLimiter::runtime(&config);

    let runtime = GatewayRuntime {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        kv: InMemoryKvStore::runtime(),
        trusted_documents,
        meter: grafbase_tracing::metrics::meter_from_global_provider(),
        hooks: HooksWasi::new(
            wasi.map(ComponentLoader::new)
                .transpose()
                .map_err(|e| crate::Error::InternalError(e.to_string()))?
                .flatten(),
        ),
        rate_limiter,
    };

    let config = config
        .try_into()
        .map_err(|err| crate::Error::InternalError(format!("Failed to generate engine Schema: {err}")))?;

    Ok(Engine::new(Arc::new(config), Some(schema_version.as_bytes()), runtime).await)
}

pub struct GatewayRuntime {
    fetcher: runtime::fetch::Fetcher,
    trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    meter: grafbase_tracing::otel::opentelemetry::metrics::Meter,
    hooks: HooksWasi,
    rate_limiter: runtime::rate_limiting::RateLimiter,
}

impl engine_v2::Runtime for GatewayRuntime {
    type Hooks = HooksWasi;
    type CacheFactory = ();

    fn fetcher(&self) -> &runtime::fetch::Fetcher {
        &self.fetcher
    }
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }
    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }
    fn meter(&self) -> &grafbase_tracing::otel::opentelemetry::metrics::Meter {
        &self.meter
    }
    fn hooks(&self) -> &HooksWasi {
        &self.hooks
    }
    fn cache_factory(&self) -> &Self::CacheFactory {
        &()
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }
}
