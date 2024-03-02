use std::{fmt::Display, sync::Arc};

use futures_util::{future::BoxFuture, Future};

pub trait AsyncRuntimeInner: Send + Sync {
    fn spawn(&self, future: BoxFuture<'static, ()>);
}

#[derive(Clone)]
pub struct AsyncRuntime(Arc<dyn AsyncRuntimeInner>);

impl AsyncRuntime {
    pub fn new(inner: impl AsyncRuntimeInner + 'static) -> Self {
        Self(Arc::new(inner))
    }

    pub fn spawn_faillible<E: Display>(&self, fut: impl Future<Output = Result<(), E>> + Send + 'static) {
        self.spawn(Box::pin(async {
            if let Err(e) = fut.await {
                tracing::error!("{}", e.to_string());
            }
        }))
    }
}

impl std::ops::Deref for AsyncRuntime {
    type Target = dyn AsyncRuntimeInner;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
