mod headers;
mod kafka_consumer;
mod kafka_producer;

use std::sync::Arc;

use crate::tonic;
use futures::StreamExt;
use runtime::extension::Token;
use sqlx::Postgres;

pub use crate::access_log::AccessLogSender;
pub use crate::context::SharedContext;
pub use headers::*;
pub use kafka_consumer::*;
pub use kafka_producer::*;

#[derive(Clone)]
pub struct SharedResources {
    pub access_log: AccessLogSender,
}

pub type GrpcClient = tonic::client::Grpc<tonic::transport::Channel>;
pub type GrpcStreamingResponse = (
    tonic::metadata::MetadataMap,
    tonic::Streaming<Vec<u8>>,
    tonic::Extensions,
);

pub type NatsClient = async_nats::Client;
pub type NatsKeyValue = async_nats::jetstream::kv::Store;

pub type PgPool = sqlx::Pool<Postgres>;
pub type PgConnection = sqlx::pool::PoolConnection<Postgres>;
pub type PgTransaction = sqlx::Transaction<'static, Postgres>;
pub type PgRow = sqlx::postgres::PgRow;

pub type EventQueue = (); // TODO

pub enum NatsSubscriber {
    Stream(Box<async_nats::jetstream::consumer::pull::Stream>),
    Subject(async_nats::Subscriber),
}

impl NatsSubscriber {
    pub async fn next(&mut self) -> Result<Option<async_nats::Message>, String> {
        match self {
            NatsSubscriber::Stream(stream) => match stream.as_mut().next().await {
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

/// It's not possible to provide a reference to wasmtime, it must be static and there are too many
/// layers to have good control over what's happening to use a transmute to get a &'static.
/// So this struct represents a lease that the engine grants on some value T that we expect to have
/// back. Depending on circumstances it may be one of the three possibilities.
pub enum Lease<T> {
    Singleton(T),
    Shared(Arc<T>),
    SharedMut(Arc<tokio::sync::RwLock<T>>),
}

impl<T> From<T> for Lease<T> {
    fn from(t: T) -> Self {
        Lease::Singleton(t)
    }
}

impl<T> Lease<T> {
    pub(crate) fn into_inner(self) -> Option<T> {
        match self {
            Lease::Singleton(t) => Some(t),
            Lease::Shared(t) => Arc::into_inner(t),
            Lease::SharedMut(t) => Arc::into_inner(t).map(|t| t.into_inner()),
        }
    }
}

impl<T> WasmOwnedOrLease<T> {
    pub(crate) fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub(crate) fn into_lease(self) -> Option<Lease<T>> {
        match self {
            Self::Lease(v) => Some(v),
            _ => None,
        }
    }

    pub(crate) async fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R
    where
        T: Send + Sync + 'static,
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

    pub(crate) async fn with_ref_mut<R>(&mut self, f: impl FnOnce(Option<&mut T>) -> R) -> R
    where
        T: Send + Sync + 'static,
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
