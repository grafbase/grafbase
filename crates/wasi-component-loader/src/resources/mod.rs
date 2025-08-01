mod cache;
mod file_logger;
mod headers;
mod kafka_consumer;
mod kafka_producer;
mod legacy_sdk18;
mod nats;

use std::sync::Arc;

use sqlx::Postgres;

pub use crate::context::WasmContext;
pub use cache::*;
pub use headers::*;
pub use kafka_consumer::*;
pub use kafka_producer::*;
pub use legacy_sdk18::*;
pub use nats::*;

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
pub type FileLogger = file_logger::FileLogger;

pub struct EventQueueProxy(pub(crate) WasmContext);
pub type AccessLogSender = ();

/// It's not possible to provide a reference to wasmtime, it must be static and there are too many
/// layers to have good control over what's happening to use a transmute to get a &'static.
/// So this struct represents a lease that the engine grants on some value T that we expect to have
/// back. Depending on circumstances it may be one of the three possibilities.
pub enum OwnedOrShared<T> {
    Owned(T),
    Shared(Arc<T>),
    SharedMut(Arc<tokio::sync::RwLock<T>>),
}

impl<T> OwnedOrShared<T> {
    pub(crate) fn into_inner(self) -> Option<T> {
        match self {
            Self::Owned(v) => Some(v),
            Self::Shared(t) => Arc::into_inner(t),
            Self::SharedMut(t) => Arc::into_inner(t).map(|t| t.into_inner()),
        }
    }

    pub(crate) async fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Self::Owned(v) => v,
            Self::Shared(v) => v.as_ref(),
            Self::SharedMut(v) => {
                _guard = Some(v.read().await);
                _guard.as_deref().unwrap()
            }
        };
        f(v)
    }

    pub(crate) async fn with_ref_mut<R>(&mut self, f: impl FnOnce(Option<&mut T>) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Self::Owned(v) => Some(v),
            Self::Shared(_) => None,
            Self::SharedMut(v) => {
                _guard = Some(v.write().await);
                _guard.as_deref_mut()
            }
        };
        f(v)
    }
}

impl<T> From<T> for OwnedOrShared<T> {
    fn from(v: T) -> Self {
        Self::Owned(v)
    }
}

impl<T> From<Arc<T>> for OwnedOrShared<T> {
    fn from(v: Arc<T>) -> Self {
        Self::Shared(v)
    }
}
