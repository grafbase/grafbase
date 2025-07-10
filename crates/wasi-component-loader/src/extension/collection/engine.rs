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
    subscriptions: Subscriptions,
}

impl EngineWasmExtensions {
    pub async fn new(
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        schema: &Arc<Schema>,
        logging_filter: String,
    ) -> crate::Result<Self> {
        let extensions = load_extensions_config(extension_catalog, gateway_config, logging_filter, |ty| {
            matches!(
                ty,
                TypeDiscriminants::Resolver
                    | TypeDiscriminants::FieldResolver
                    | TypeDiscriminants::SelectionSetResolver
                    | TypeDiscriminants::Authorization
                    | TypeDiscriminants::Authentication
            )
        });

        Ok(Self(Arc::new(EngineWasmExtensionsInner {
            pools: create_pools(schema, extensions).await?,
            subscriptions: Default::default(),
        })))
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

        let pool = Pool::new(schema.clone(), &config).await?;

        crate::Result::Ok((config.id, pool))
    }))
    .buffer_unordered(parallelism)
    .try_collect()
    .await
}
