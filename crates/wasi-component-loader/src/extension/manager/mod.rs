mod config;
mod instance;
mod loader;
mod pool;

use crate::resources::SharedResources;

use dashmap::DashMap;
use engine_error::GraphqlError;
use engine_schema::Schema;
use extension_catalog::{ExtensionCatalog, ExtensionId};
use futures::TryStreamExt;
use futures_util::{StreamExt, stream};
use gateway_config::Config;
use runtime::extension::Data;
use std::sync::Arc;
use tokio::sync::broadcast;

pub(crate) use config::*;
pub(crate) use instance::*;
pub(crate) use loader::*;
pub(crate) use pool::*;

#[derive(Clone, Default)]
pub struct WasmExtensions(Arc<WasiExtensionsInner>);

pub(crate) type Subscriptions = DashMap<Vec<u8>, broadcast::Sender<Result<Arc<Data>, GraphqlError>>>;

#[derive(Default)]
struct WasiExtensionsInner {
    // Indexed by ExtensionId
    instance_pools: Vec<Pool>,
    subscriptions: Subscriptions,
}

impl WasmExtensions {
    pub async fn new(
        shared_resources: SharedResources,
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        schema: &Schema,
    ) -> crate::Result<Self> {
        let extensions = config::load_extensions_config(extension_catalog, gateway_config, schema);
        Ok(Self(Arc::new(WasiExtensionsInner {
            instance_pools: create_pools(&shared_resources, extensions).await?,
            subscriptions: Default::default(),
        })))
    }

    pub(super) async fn get(&self, id: ExtensionId) -> Result<ExtensionGuard, GraphqlError> {
        let pool = self
            .0
            .as_ref()
            .instance_pools
            .get(usize::from(id))
            .ok_or_else(GraphqlError::internal_extension_error)?;
        pool.get().await.map_err(|err| {
            tracing::error!("Failed to retrieve extension: {err}");
            GraphqlError::internal_extension_error()
        })
    }

    pub(super) fn subscriptions(&self) -> &Subscriptions {
        &self.0.subscriptions
    }
}

async fn create_pools(
    shared_resources: &SharedResources,
    extensions: Vec<ExtensionConfig>,
) -> crate::Result<Vec<Pool>> {
    let parallelism = std::thread::available_parallelism()
        .ok()
        // Each extensions takes quite a lot of CPU.
        .map(|num| num.get() / 8)
        .unwrap_or(1);

    let mut pools = stream::iter(extensions.into_iter().map(|config| async {
        let shared = shared_resources.clone();
        std::future::ready(()).await;

        tracing::info!("Loading extension {}", config.manifest_id);

        let id = config.id;
        let max_pool_size = config.pool.max_size;
        ExtensionLoader::new(shared, config).map(|loader| (id, Pool::new(loader, max_pool_size)))
    }))
    .buffer_unordered(parallelism)
    .try_collect::<Vec<_>>()
    .await?;

    pools.sort_by_key(|(id, _)| *id);
    Ok(pools.into_iter().map(|(_, pool)| pool).collect())
}
