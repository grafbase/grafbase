use std::ops::{Deref, DerefMut};

use deadpool::managed::{self, Manager};
use tracing::{Instrument, info_span};

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
    pub(super) fn new(loader: ExtensionLoader, size: Option<usize>) -> Self {
        let mut builder = managed::Pool::builder(loader);

        if let Some(size) = size {
            builder = builder.max_size(size);
        }

        let inner = builder.build().expect("only fails if not in a runtime");

        Pool { inner }
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
