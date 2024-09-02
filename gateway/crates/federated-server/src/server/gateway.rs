use crate::hot_reload::ConfigWatcher;
use engine_v2::Engine;
use gateway_config::{Config, EntityCachingRedisConfig};
use grafbase_telemetry::metrics::EngineMetrics;
use graphql_composition::VersionedFederatedGraph;
use runtime::entity_cache::EntityCache;
use runtime_local::{
    rate_limiting::{in_memory::key_based::InMemoryRateLimiter, redis::RedisRateLimiter},
    redis::{RedisPoolFactory, RedisTlsConfig},
    HooksWasi, InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCacheFactory, NativeFetcher, RedisEntityCache,
};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::watch;

use super::GdnResponse;

/// Send half of the gateway watch channel
pub(crate) type GatewaySender = watch::Sender<Option<Arc<Engine<GatewayRuntime>>>>;

/// Receive half of the gateway watch channel.
///
/// Anything part of the system that needs access to the gateway can use this
pub(crate) type EngineWatcher = watch::Receiver<Option<Arc<Engine<GatewayRuntime>>>>;

pub(crate) enum GraphDefinition {
    Gdn(GdnResponse),
    Sdl(String),
}

/// Creates a new gateway from federated schema.
pub(super) async fn generate(
    graph_definition: GraphDefinition,
    gateway_config: &Config,
    hot_reload_config_path: Option<PathBuf>,
    hooks: HooksWasi,
) -> crate::Result<Engine<GatewayRuntime>> {
    let (federated_sdl, schema_version, version_id, trusted_documents) = match graph_definition {
        GraphDefinition::Gdn(GdnResponse {
            branch_id,
            sdl,
            version_id,
            ..
        }) => {
            let version = engine_v2::SchemaVersion::from(
                [b"id:".to_vec(), version_id.to_bytes().to_vec()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<u8>>(),
            );

            let trusted_documents = if gateway_config.trusted_documents.enabled {
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
                    ),
                ))
            } else {
                None
            };

            (sdl, version, Some(version_id), trusted_documents)
        }
        GraphDefinition::Sdl(federated_sdl) => {
            let version = engine_v2::SchemaVersion::from(
                [
                    b"hash:".to_vec(),
                    blake3::hash(federated_sdl.as_bytes()).as_bytes().to_vec(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>(),
            );
            // TODO: https://linear.app/grafbase/issue/GB-6168/support-trusted-documents-in-air-gapped-mode
            (federated_sdl, version, None, None)
        }
    };

    let config = {
        let graph = VersionedFederatedGraph::from_sdl(&federated_sdl)
            .map_err(|e| crate::Error::SchemaValidationError(e.to_string()))?;

        engine_config_builder::build_with_toml_config(gateway_config, graph.into_latest()).into_latest()
    };

    let mut runtime = GatewayRuntime::build(gateway_config, hot_reload_config_path, &config, version_id, hooks).await?;

    if let Some(trusted_documents) = trusted_documents {
        runtime.trusted_documents = trusted_documents;
    }

    let schema = engine_v2::Schema::build(config, schema_version).map_err(|err| match err {
        err @ engine_v2::BuildError::RequiredFieldArgumentCoercionError { .. } => {
            crate::Error::InternalError(format!("Failed to generate engine Schema: {err}"))
        }
        engine_v2::BuildError::GraphFromSdlError(err) => crate::Error::SchemaValidationError(err.to_string()),
    })?;

    Ok(Engine::new(Arc::new(schema), runtime).await)
}

pub struct GatewayRuntime {
    fetcher: NativeFetcher,
    trusted_documents: runtime::trusted_documents_client::Client,
    kv: runtime::kv::KvStore,
    metrics: EngineMetrics,
    hooks: HooksWasi,
    rate_limiter: runtime::rate_limiting::RateLimiter,
    entity_cache: Box<dyn EntityCache>,
    operation_cache_factory: InMemoryOperationCacheFactory,
}

impl GatewayRuntime {
    async fn build(
        gateway_config: &Config,
        hot_reload_config_path: Option<PathBuf>,
        config: &engine_v2::config::Config,
        version_id: Option<ulid::Ulid>,
        hooks: HooksWasi,
    ) -> Result<GatewayRuntime, crate::Error> {
        let mut redis_factory = RedisPoolFactory::default();
        let watcher = ConfigWatcher::init(gateway_config.clone(), hot_reload_config_path)?;
        let meter = grafbase_telemetry::metrics::meter_from_global_provider();
        let rate_limiter = match config.rate_limit_config() {
            Some(config) if config.storage.is_redis() => {
                let tls = config.redis.tls.map(|tls| RedisTlsConfig {
                    cert: tls.cert,
                    key: tls.key,
                    ca: tls.ca,
                });

                let pool = redis_factory
                    .pool(config.redis.url, tls)
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                let global_config = runtime_local::rate_limiting::redis::RateLimitRedisConfig {
                    key_prefix: config.redis.key_prefix,
                };

                RedisRateLimiter::runtime(global_config, pool, watcher, &meter)
                    .await
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?
            }
            _ => InMemoryRateLimiter::runtime_with_watcher(watcher),
        };
        let entity_cache: Box<dyn EntityCache> = match gateway_config.entity_caching.storage {
            gateway_config::EntityCachingStorage::Memory => Box::new(InMemoryEntityCache::default()),
            gateway_config::EntityCachingStorage::Redis => {
                let EntityCachingRedisConfig { url, key_prefix, tls } = &gateway_config.entity_caching.redis;
                let tls = tls.as_ref().map(|tls| RedisTlsConfig {
                    cert: tls.cert.as_deref(),
                    key: tls.key.as_deref(),
                    ca: tls.ca.as_deref(),
                });

                let pool = redis_factory
                    .pool(url.as_str(), tls)
                    .map_err(|e| crate::Error::InternalError(e.to_string()))?;

                Box::new(RedisEntityCache::new(pool, key_prefix))
            }
        };

        let runtime = GatewayRuntime {
            fetcher: NativeFetcher::default(),
            kv: InMemoryKvStore::runtime(),
            trusted_documents: runtime::trusted_documents_client::Client::new(NoopTrustedDocuments),
            hooks,
            metrics: EngineMetrics::build(&meter, version_id.map(|id| id.to_string())),
            rate_limiter,
            entity_cache,
            operation_cache_factory: InMemoryOperationCacheFactory::default(),
        };

        Ok(runtime)
    }
}

impl engine_v2::Runtime for GatewayRuntime {
    type Hooks = HooksWasi;
    type Fetcher = NativeFetcher;
    type OperationCacheFactory = InMemoryOperationCacheFactory;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn hooks(&self) -> &HooksWasi {
        &self.hooks
    }

    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory {
        &self.operation_cache_factory
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await
    }

    fn entity_cache(&self) -> &dyn EntityCache {
        self.entity_cache.as_ref()
    }

    fn metrics(&self) -> &grafbase_telemetry::metrics::EngineMetrics {
        &self.metrics
    }
}
