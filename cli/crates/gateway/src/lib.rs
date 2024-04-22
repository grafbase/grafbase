use auth::AnyApiKeyProvider;
use engine::registry::CachePartialRegistry;
use gateway_core::CacheConfig;
use runtime_local::{InMemoryCache, InMemoryKvStore};
use std::{collections::HashMap, ops::Deref, sync::Arc};

use self::executor::Executor;

mod auth;
mod context;
mod error;
mod executor;
mod response;
mod serving;

pub(crate) use context::Context;
pub(crate) use error::Error;
pub(crate) use response::Response;
pub use runtime_local::Bridge;

pub type GatewayInner = gateway_core::Gateway<Executor>;

#[derive(Clone)]
pub struct Gateway {
    inner: Arc<GatewayInner>,
}

impl Gateway {
    pub async fn new(
        env_vars: HashMap<String, String>,
        bridge: Bridge,
        registry: Arc<registry_v2::Registry>,
    ) -> Result<Self, crate::Error> {
        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: "localhost".to_string(),
            host_name: "localhost".to_string(),
            partial_registry: CachePartialRegistry::default(), // TODO: Revisit this /*CachePartialRegistry::from(registry.as_ref()),*/
            common_cache_tags: vec![],
        };
        let authorizer = Box::new(auth::Authorizer);
        let auth = gateway_v2_auth::AuthService::new_v1(
            registry.auth.clone(),
            InMemoryKvStore::runtime(),
            runtime_local::UdfInvokerImpl::authorizer(bridge.clone()),
            String::new(),
        )
        .with_first_authorizer(AnyApiKeyProvider);

        let executor = Arc::new(Executor::new(env_vars, bridge, registry).await?);
        let trusted_documents =
            runtime::trusted_documents_client::Client::new(runtime_noop::trusted_documents::NoopTrustedDocuments);

        Ok(Gateway {
            inner: Arc::new(gateway_core::Gateway::new(
                executor,
                InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
                    common_cache_tags: vec![],
                    enabled: true,
                    subdomain: "localhost".to_string(),
                }),
                cache_config,
                auth,
                authorizer,
                trusted_documents,
            )),
        })
    }

    pub fn into_router(self) -> axum::Router {
        serving::router(self)
    }
}

impl Deref for Gateway {
    type Target = GatewayInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
