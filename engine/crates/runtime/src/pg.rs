use std::ops::Deref;

use postgres_connector_types::{database_definition::DatabaseDefinition, transport::Transport};

#[derive(Debug, thiserror::Error)]
pub enum PgTransportFactoryError {
    #[error("Transport creation error: {0}")]
    TransportCreation(#[from] postgres_connector_types::error::Error),
}

pub type PgTransportFactoryResult<T> = std::result::Result<T, PgTransportFactoryError>;

#[async_trait::async_trait]
pub trait PgTransportFactoryInner {
    async fn try_new(
        &self,
        name: &str,
        database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>>;
}

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
