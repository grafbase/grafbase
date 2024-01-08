use engine::registry::CachePartialRegistry;
use gateway_core::CacheConfig;
use runtime::cache::RequestCacheControl;
use runtime_local::InMemoryCache;
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

pub type GatewayInner = gateway_core::Gateway<Executor, InMemoryCache<engine::Response>>;

#[derive(Clone)]
pub struct Gateway {
    inner: Arc<GatewayInner>,
}

impl Gateway {
    pub async fn new(
        env_vars: HashMap<String, String>,
        bridge: Bridge,
        registry: Arc<engine::Registry>,
    ) -> Result<Self, crate::Error> {
        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: "localhost".to_string(),
            host_name: "localhost".to_string(),
            request_cache_control: RequestCacheControl::default(),
            partial_registry: CachePartialRegistry::from(registry.as_ref()),
            common_cache_tags: vec![],
        };
        let authorizer = Box::new(auth::Authorizer {
            auth_config: registry.auth.clone(),
            bridge: bridge.clone(),
        });

        let executor = Arc::new(Executor::new(env_vars, bridge, registry).await?);

        Ok(Gateway {
            inner: Arc::new(gateway_core::Gateway::new(
                executor,
                Arc::new(InMemoryCache::<engine::Response>::new()),
                cache_config,
                authorizer,
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
