mod config;
mod instance;
mod loader;
mod pool;

use crate::resources::SharedResources;

use dashmap::DashMap;
use engine::GraphqlError;
use extension_catalog::{ExtensionCatalog, ExtensionId};
use futures_util::{StreamExt, stream};
use gateway_config::Config;
use runtime::extension::{AuthorizerId, Data};
use semver::Version;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{sync::broadcast, task::JoinHandle};

pub(crate) use instance::*;
pub(crate) use loader::*;
pub(crate) use pool::*;

#[derive(Clone, Default)]
pub struct WasmExtensions(Arc<WasiExtensionsInner>);

pub(crate) type Subscriptions = DashMap<Vec<u8>, broadcast::Sender<Result<Arc<Data>, GraphqlError>>>;

#[derive(Default)]
struct WasiExtensionsInner {
    instance_pools: HashMap<ExtensionPoolId, Pool>,
    subscriptions: Subscriptions,
}

impl WasmExtensions {
    pub async fn new(
        shared_resources: SharedResources,
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        schema: &engine::Schema,
    ) -> crate::Result<Self> {
        let extensions = config::load_extensions_config(extension_catalog, gateway_config, schema);
        if extensions.is_empty() {
            return Ok(Default::default());
        }

        let instance_pools = create_pools(shared_resources, extensions).await?;

        let inner = WasiExtensionsInner {
            instance_pools,
            subscriptions: Default::default(),
        };

        Ok(Self(Arc::new(inner)))
    }

    pub(super) async fn get(&self, id: ExtensionPoolId) -> Result<ExtensionGuard, GraphqlError> {
        let pool = self
            .0
            .as_ref()
            .instance_pools
            .get(&id)
            .ok_or_else(GraphqlError::internal_extension_error)?;
        Ok(pool.get().await)
    }

    pub(super) fn subscriptions(&self) -> &Subscriptions {
        &self.0.subscriptions
    }
}

async fn create_pools<T: serde::Serialize + Send + 'static>(
    shared_resources: SharedResources,
    extensions: Vec<ExtensionConfig<T>>,
) -> crate::Result<HashMap<ExtensionPoolId, Pool>> {
    type Handle = JoinHandle<crate::Result<(ExtensionPoolId, Pool)>>;

    let mut creating_pools: Vec<Handle> = Vec::new();

    for config in extensions {
        let shared = shared_resources.clone();

        creating_pools.push(tokio::task::spawn_blocking(move || {
            tracing::info!("Loading extension {}", config.manifest_id);

            let loader = ExtensionLoader::new(shared, config.wasm_config, config.guest_config, config.sdk_version)?;

            Ok((config.id, Pool::new(loader, config.max_pool_size)))
        }));
    }

    let mut pools = HashMap::new();

    let mut creating_pools = stream::iter(creating_pools)
        .buffer_unordered(std::thread::available_parallelism().map(|i| i.get()).unwrap_or(1));

    while let Some(result) = creating_pools.next().await {
        match result.unwrap() {
            Ok((id, pool)) => {
                pools.insert(id, pool);
            }
            Err(error) => return Err(error),
        }
    }

    Ok(pools)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionPoolId {
    Resolver(ExtensionId),
    Authorizer(ExtensionId, AuthorizerId),
    Authorization(ExtensionId),
}

impl From<ExtensionId> for ExtensionPoolId {
    fn from(id: ExtensionId) -> Self {
        Self::Resolver(id)
    }
}

impl From<(ExtensionId, AuthorizerId)> for ExtensionPoolId {
    fn from((id, authorizer_id): (ExtensionId, AuthorizerId)) -> Self {
        Self::Authorizer(id, authorizer_id)
    }
}

pub struct ExtensionConfig<T> {
    pub id: ExtensionPoolId,
    pub manifest_id: extension_catalog::Id,
    pub max_pool_size: Option<usize>,
    pub wasm_config: WasmConfig,
    pub guest_config: ExtensionGuestConfig<T>,
    pub sdk_version: Version,
}

#[derive(Debug, Clone)]
pub struct WasmConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
}
