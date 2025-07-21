use std::sync::Arc;

use deadpool::managed::{self, Manager};
use engine_error::{ErrorResponse, GraphqlError};
use engine_schema::Schema;
use extension_catalog::ExtensionId;
use runtime::extension::Response;
use tracing::{Instrument, info_span};

use crate::{WasiState, extension::ExtensionConfig};

use super::{ExtensionInstance, ExtensionLoader};

pub(crate) struct Pool {
    inner: managed::Pool<ExtensionLoader>,
}

impl Pool {
    pub(super) async fn new(schema: Arc<Schema>, config: Arc<ExtensionConfig>) -> wasmtime::Result<Self> {
        let loader = ExtensionLoader::new(schema, config.clone())?;
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

    pub(crate) async fn get(&self) -> wasmtime::Result<ExtensionGuard> {
        let span = info_span!("get extension from pool");

        let instance = self.inner.get().instrument(span).await.map_err(|err| match err {
            managed::PoolError::Backend(err) => err,
            err => wasmtime::Error::msg(err),
        })?;

        Ok(ExtensionGuard(instance))
    }

    pub(crate) fn id(&self) -> ExtensionId {
        self.inner.manager().config.id
    }

    pub(crate) async fn clone_and_adjust_for_contract(&self, schema: &Arc<Schema>) -> wasmtime::Result<Self> {
        let config = self.inner.manager().config.clone();
        Self::new(schema.clone(), config).await
    }
}

impl Manager for ExtensionLoader {
    type Type = Instance;
    type Error = wasmtime::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(Instance {
            inner: self.instantiate().await?,
            poisoned: false,
            recyclable: true,
        })
    }

    async fn recycle(
        &self,
        instance: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if instance.poisoned || !instance.recyclable {
            if instance.poisoned {
                return Err(managed::RecycleError::Message("Poisonned".into()));
            } else {
                return Err(managed::RecycleError::Message("Not recyclable".into()));
            }
        }

        Ok(())
    }
}

pub(crate) struct Instance {
    inner: Box<dyn ExtensionInstance>,
    pub poisoned: bool,
    pub recyclable: bool,
}

pub(crate) struct ExtensionGuard(managed::Object<ExtensionLoader>);

impl std::ops::Deref for ExtensionGuard {
    type Target = Instance;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl std::ops::DerefMut for ExtensionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

// Getting lifetime issues with a closure, so just doing a macro...
#[macro_export]
macro_rules! wasmsafe {
    ($instance:ident . $($call:tt)*) => {{
        debug_assert!(
            !$instance.poisoned,
            "ExtensionGuard is poisoned, cannot call methods on it."
        );
        // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
        // otherwise. If there is any wasmtime error we also assume the instance to be poisoned and
        // unrecoverable.
        $instance.poisoned = true;
        match $instance.dont_use_me_without_wasmsafe().$($call)* {
            Ok(result) => {
                $instance.poisoned = false; // Reset poisoned state if the call was successful.
                result
            }
            Err(err) => {
                $crate::extension::pool::FromWasmtimeError::from_wasmtime_error(err)
            }
        }
    }};
}

impl ExtensionGuard {
    pub fn store(&self) -> &wasmtime::Store<WasiState> {
        self.0.inner.store()
    }

    pub fn dont_use_me_without_wasmsafe(&mut self) -> &mut dyn ExtensionInstance {
        self.0.inner.as_mut()
    }
}

pub(crate) trait FromWasmtimeError {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self;
}

impl FromWasmtimeError for Response {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self {
        tracing::error!("Wasm error: {err}");
        Response {
            data: None,
            errors: vec![GraphqlError::internal_extension_error()],
        }
    }
}

impl FromWasmtimeError for Result<(), wasmtime::Error> {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self {
        Err(err)
    }
}

impl<T> FromWasmtimeError for Result<T, GraphqlError> {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self {
        tracing::error!("Wasm error: {err}");
        Err(GraphqlError::internal_extension_error())
    }
}

impl<T> FromWasmtimeError for Result<T, ErrorResponse> {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self {
        tracing::error!("Wasm error: {err}");
        Err(ErrorResponse::internal_extension_error())
    }
}

impl<T> FromWasmtimeError for Result<T, String> {
    fn from_wasmtime_error(err: wasmtime::Error) -> Self {
        Err(err.to_string())
    }
}
