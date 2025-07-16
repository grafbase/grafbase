use dashmap::DashMap;
use engine_error::GraphqlError;
use engine_schema::Schema;
use extension_catalog::{ExtensionCatalog, ExtensionId, TypeDiscriminants};
use futures::TryStreamExt;
use futures_util::{StreamExt, stream};
use fxhash::FxHasher32;
use gateway_config::Config;
use runtime::extension::Response;
use std::{collections::HashMap, hash::BuildHasherDefault, sync::Arc};
use tokio::sync::broadcast;

use crate::extension::{ExtensionConfig, ExtensionGuard, Pool, load_extensions_config};

/// Extensions tied to the life cycle of the engine. Whenever it reloads, they must also be
/// reloaded.
#[derive(Clone, Default)]
pub struct EngineWasmExtensions(Arc<EngineWasmExtensionsInner>);

pub(crate) type Subscriptions = DashMap<Vec<u8>, broadcast::Sender<Response>>;

#[derive(Default)]
struct EngineWasmExtensionsInner {
    pools: HashMap<ExtensionId, Pool, BuildHasherDefault<FxHasher32>>,
    contracts: Option<Pool>,
    subscriptions: Subscriptions,
}

impl EngineWasmExtensions {
    pub async fn new(
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        schema: &Arc<Schema>,
        logging_filter: String,
    ) -> crate::Result<Self> {
        let mut extensions = load_extensions_config(extension_catalog, gateway_config, logging_filter, |ty| {
            matches!(
                ty,
                TypeDiscriminants::Resolver
                    | TypeDiscriminants::FieldResolver
                    | TypeDiscriminants::SelectionSetResolver
                    | TypeDiscriminants::Authorization
            )
        });

        let contracts = extensions
            .iter()
            .position(|ext| matches!(ext.r#type, TypeDiscriminants::Contracts))
            .map(|i| extensions.swap_remove(i));

        if extensions
            .iter()
            .any(|ext| matches!(ext.r#type, TypeDiscriminants::Contracts))
        {
            return Err(crate::Error::Internal(anyhow::anyhow!(
                "Multiple contracts extensions cannot be used together."
            )));
        }

        Ok(Self(Arc::new(EngineWasmExtensionsInner {
            pools: create_pools(schema, extensions).await?,
            contracts: match contracts {
                Some(config) => Some(Pool::new(schema.clone(), Arc::new(config)).await?),
                None => None,
            },
            subscriptions: Default::default(),
        })))
    }

    pub(crate) async fn contracts(&self) -> Result<Option<ExtensionGuard>, GraphqlError> {
        if let Some(pool) = self.0.contracts.as_ref() {
            pool.get()
                .await
                .map_err(|err| {
                    tracing::error!("Failed to retrieve extension: {err}");
                    GraphqlError::internal_extension_error()
                })
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) async fn get(&self, id: ExtensionId) -> Result<ExtensionGuard, GraphqlError> {
        let pool = self
            .0
            .as_ref()
            .pools
            .get(&id)
            .ok_or_else(GraphqlError::internal_extension_error)?;

        pool.get().await.map_err(|err| {
            tracing::error!("Failed to retrieve extension: {err}");
            GraphqlError::internal_extension_error()
        })
    }

    pub(crate) fn subscriptions(&self) -> &Subscriptions {
        &self.0.subscriptions
    }

    pub async fn clone_and_adjust_for_contract(&self, schema: &Arc<Schema>) -> crate::Result<Self> {
        let mut pools =
            HashMap::with_capacity_and_hasher(self.0.pools.len(), BuildHasherDefault::<FxHasher32>::default());
        for (id, pool) in self.0.pools.iter() {
            let pool = pool.clone_and_adjust_for_contract(schema).await?;
            pools.insert(*id, pool);
        }
        Ok(Self(Arc::new(EngineWasmExtensionsInner {
            pools,
            contracts: None,
            subscriptions: Default::default(),
        })))
    }
}

async fn create_pools(
    schema: &Arc<Schema>,
    extensions: Vec<ExtensionConfig>,
) -> crate::Result<HashMap<ExtensionId, Pool, BuildHasherDefault<FxHasher32>>> {
    let parallelism = std::thread::available_parallelism()
        .ok()
        // Each extensions takes quite a lot of CPU.
        .map(|num| num.get() / 8)
        .unwrap_or_default()
        // We want at least parallelism of 1, otherwise we'll never move forward even without any
        // extensions...
        .max(1);

    stream::iter(extensions.into_iter().map(|config| async move {
        tracing::info!("Loading extension {}", config.manifest_id);

        std::future::ready(()).await;

        let id = config.id;
        let pool = Pool::new(schema.clone(), Arc::new(config)).await?;

        crate::Result::Ok((id, pool))
    }))
    .buffer_unordered(parallelism)
    .try_collect()
    .await
}
