mod config;
mod hooks;
mod instance;
mod loader;
mod pool;

use dashmap::DashMap;
use engine_error::GraphqlError;
use engine_schema::Schema;
use extension_catalog::{ExtensionCatalog, ExtensionId};
use futures::TryStreamExt;
use futures_util::{StreamExt, stream};
use gateway_config::Config;
use runtime::extension::Response;
use std::sync::Arc;
use tokio::sync::broadcast;

pub use hooks::*;

pub(crate) use config::*;
pub(crate) use instance::*;
pub(crate) use loader::*;
pub(crate) use pool::*;

#[derive(Clone, Default)]
pub struct WasmExtensions(Arc<WasiExtensionsInner>);

pub(crate) type Subscriptions = DashMap<Vec<u8>, broadcast::Sender<Response>>;

#[derive(Default)]
struct WasiExtensionsInner {
    // Indexed by ExtensionId
    instance_pools: Vec<Pool>,
    subscriptions: Subscriptions,
}

impl WasmExtensions {
    pub async fn new(
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        schema: &Arc<Schema>,
    ) -> crate::Result<Self> {
        // FIXME: Rely on hook configuration to define whether events can be skipped or not
        let can_skip_sending_events = false;
        let extensions = config::load_extensions_config(extension_catalog, gateway_config, can_skip_sending_events);

        Ok(Self(Arc::new(WasiExtensionsInner {
            instance_pools: create_pools(schema, extensions).await?,
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

async fn create_pools(schema: &Arc<Schema>, extensions: Vec<ExtensionConfig>) -> crate::Result<Vec<Pool>> {
    let parallelism = std::thread::available_parallelism()
        .ok()
        // Each extensions takes quite a lot of CPU.
        .map(|num| num.get() / 8)
        .unwrap_or_default()
        // We want at least parallelism of 1, otherwise we'll never move forward even without any
        // extensions...
        .max(1);

    let mut pools = stream::iter(extensions.into_iter().map(|config| async {
        tracing::info!("Loading extension {}", config.manifest_id);

        std::future::ready(()).await;

        let id = config.id;
        let max_pool_size = config.pool.max_size;
        let loader = ExtensionLoader::new(Arc::clone(schema), config)?;
        let pool = Pool::new(loader, max_pool_size);

        // Load immediately an instance to check they can initialize themselves correctly.
        let _ = pool.get().await?;

        crate::Result::Ok((id, pool))
    }))
    .buffer_unordered(parallelism)
    .try_collect::<Vec<_>>()
    .await?;

    pools.sort_by_key(|(id, _)| *id);
    Ok(pools.into_iter().map(|(_, pool)| pool).collect())
}
