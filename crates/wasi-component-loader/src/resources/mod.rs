mod headers;

use futures::StreamExt;
use runtime::extension::{Lease, Token};

pub use crate::access_log::AccessLogSender;
pub use crate::context::SharedContext;
pub use headers::*;

#[derive(Clone)]
pub struct SharedResources {
    pub access_log: AccessLogSender,
}

pub type NatsClient = async_nats::Client;
pub type NatsKeyValue = async_nats::jetstream::kv::Store;

pub enum NatsSubscriber {
    Stream(async_nats::jetstream::consumer::pull::Stream),
    Subject(async_nats::Subscriber),
}

impl NatsSubscriber {
    pub async fn next(&mut self) -> Result<Option<async_nats::Message>, String> {
        match self {
            NatsSubscriber::Stream(stream) => match stream.next().await {
                Some(Ok(message)) => Ok(Some(message.into())),
                Some(Err(err)) => Err(err.to_string()),
                None => Ok(None),
            },
            NatsSubscriber::Subject(subject) => Ok(subject.next().await),
        }
    }
}

pub struct AuthorizationContext {
    pub headers: WasmOwnedOrLease<http::HeaderMap>,
    pub token: Token,
}

pub enum WasmOwnedOrLease<T> {
    Owned(T),
    Lease(Lease<T>),
}

impl<T> WasmOwnedOrLease<T> {
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub fn into_lease(self) -> Option<Lease<T>> {
        match self {
            Self::Lease(v) => Some(v),
            _ => None,
        }
    }

    pub async fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R
    where
        T: Send + Sync,
    {
        let mut _guard = None;
        let v = match self {
            Self::Lease(Lease::Shared(v)) => v.as_ref(),
            Self::Lease(Lease::SharedMut(v)) => {
                _guard = Some(v.read().await);
                _guard.as_deref().unwrap()
            }
            Self::Lease(Lease::Singleton(v)) => v,
            Self::Owned(v) => v,
        };
        f(v)
    }

    pub async fn with_ref_mut<R>(&mut self, f: impl FnOnce(Option<&mut T>) -> R) -> R
    where
        T: Send + Sync,
    {
        let mut _guard = None;
        let v = match self {
            Self::Lease(Lease::Shared(_)) => None,
            Self::Lease(Lease::SharedMut(v)) => {
                _guard = Some(v.write().await);
                _guard.as_deref_mut()
            }
            Self::Lease(Lease::Singleton(v)) => Some(v),
            Self::Owned(v) => Some(v),
        };
        f(v)
    }
}

impl<T> From<T> for WasmOwnedOrLease<T> {
    fn from(v: T) -> Self {
        Self::Owned(v)
    }
}

impl<T> From<Lease<T>> for WasmOwnedOrLease<T> {
    fn from(v: Lease<T>) -> Self {
        Self::Lease(v)
    }
}
