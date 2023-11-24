use std::{ops::Deref, sync::Arc};

use postgres_connector_types::transport::Transport;

#[derive(Debug, thiserror::Error)]
pub enum PgTransportFactoryError {
    #[error("Transport creation error: {0}")]
    TransportCreation(#[from] postgres_connector_types::error::Error),
    #[error("Transport not found for name: {0}")]
    TransportNotFound(String),
}

pub type PgTransportFactoryResult<T> = std::result::Result<T, PgTransportFactoryError>;

type BoxedPgTransportFactoryImpl = Box<dyn PgTransportFactoryInner + Send + Sync>;

pub struct PgTransportFactory {
    inner: BoxedPgTransportFactoryImpl,
}

impl PgTransportFactory {
    pub fn new(factory: BoxedPgTransportFactoryImpl) -> PgTransportFactory {
        PgTransportFactory { inner: factory }
    }
}

impl Deref for PgTransportFactory {
    type Target = BoxedPgTransportFactoryImpl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait::async_trait]
pub trait PgTransportFactoryInner {
    async fn try_get(&self, name: &str) -> PgTransportFactoryResult<Arc<dyn Transport>>;
}
