use std::path::PathBuf;
use std::sync::Arc;

use runtime_local::rate_limiting::in_memory::key_based::InMemoryRateLimiter;
use runtime_local::rate_limiting::redis::RedisRateLimiter;
use tokio::sync::watch;

use engine_v2::Engine;
use graphql_composition::FederatedGraph;
use runtime_local::{ComponentLoader, HooksWasi, InMemoryKvStore};
use runtime_noop::trusted_documents::NoopTrustedDocuments;

use gateway_config::Config;

use crate::hot_reload::ConfigWatcher;

/// Send half of the gateway watch channel
#[cfg(not(feature = "lambda"))]
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Option<Arc<Engine<GatewayRuntime>>>>;

/// Creates a new gateway from federated schema.
pub(super) async fn generate(
    federated_schema: &str,
    branch_id: Option<ulid::Ulid>,
    gateway_config: &Config,
    hot_reload_config_path: Option<PathBuf>,
) -> crate::Result<Engine<GatewayRuntime>> {
    let schema_version = blake3::hash(federated_schema.as_bytes());
    let graph =
        FederatedGraph::from_sdl(federated_schema).map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;
    let config = engine_config_builder::build_with_toml_config(gateway_config, graph).into_latest();

    // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
    let trusted_documents = if gateway_config.trusted_documents.enabled {
        let Some(branch_id) = branch_id else {
            return Err(crate::Error::InternalError(
                "Trusted documents are not implemented yet in airgapped mode".into(),
            ));
        };

        runtime::trusted_documents_client::Client::new(super::trusted_documents_client::TrustedDocumentsClient {
            http_client: Default::default(),
            bypass_header: gateway_config
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
            branch_id,
        })
    } else {
        runtime::trusted_documents_client::Client::new(NoopTrustedDocuments)
    };

    let watcher = ConfigWatcher::init(gateway_config.clone(), hot_reload_config_path)?;

    let rate_limiter = match &gateway_config.gateway.rate_limit {
        Some(config) if config.storage.is_redis() => RedisRateLimiter::runtime(&config.redis, watcher)
            .await
            .map_err(|e| crate::Error::InternalError(e.to_string()))?,
        _ => InMemoryRateLimiter::runtime_with_watcher(watcher),
    };

    let runtime = GatewayRuntime {
        fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
        kv: InMemoryKvStore::runtime(),
        trusted_documents,
        meter: grafbase_telemetry::metrics::meter_from_global_provider(),
        hooks: HooksWasi::new(
            gateway_config
                .hooks
                .clone()
                .map(ComponentLoader::new)
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
    meter: grafbase_telemetry::otel::opentelemetry::metrics::Meter,
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
    fn meter(&self) -> &grafbase_telemetry::otel::opentelemetry::metrics::Meter {
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

    fn sleep(&self, duration: std::time::Duration) -> futures_util::future::BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(duration))
    }
}
