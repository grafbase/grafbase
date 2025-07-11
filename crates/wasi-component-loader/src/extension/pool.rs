use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use deadpool::managed::{self, Manager};
use engine_schema::Schema;
use tracing::{Instrument, info_span};

use crate::extension::ExtensionConfig;

use super::{ExtensionInstance, ExtensionLoader};

pub(crate) struct Pool {
    inner: managed::Pool<ExtensionLoader>,
}

pub(crate) struct ExtensionGuard {
    inner: managed::Object<ExtensionLoader>,
}

impl Deref for ExtensionGuard {
    type Target = Box<dyn ExtensionInstance>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ExtensionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Pool {
    pub(super) async fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
    ) -> crate::Result<Self> {
        let loader = ExtensionLoader::new(schema, config)?;
        let mut builder = managed::Pool::builder(loader);

        if let Some(size) = config.pool.max_size {
            builder = builder.max_size(size);
        }

        let inner = builder.build().expect("only fails if not in a runtime");
        let pool = Pool { inner };

        // Load immediately an instance to check they can initialize themselves correctly.
        let _ = pool.get().await?;

        Ok(pool)
    }

    pub(crate) async fn get(&self) -> crate::Result<ExtensionGuard> {
        let span = info_span!("get extension from pool");

        let inner = self.inner.get().instrument(span).await.map_err(|err| match err {
            managed::PoolError::Backend(err) => err,
            err => crate::Error::Internal(err.into()),
        })?;

        Ok(ExtensionGuard { inner })
    }
}

impl Manager for ExtensionLoader {
    type Type = Box<dyn ExtensionInstance>;
    type Error = crate::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        self.instantiate().await
    }

    async fn recycle(
        &self,
        instance: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if let Err(e) = instance.recycle() {
            return Err(e.into());
        }

        Ok(())
    }
}
