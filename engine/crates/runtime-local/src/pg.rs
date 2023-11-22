use std::{collections::HashMap, sync::Arc};

use postgres_connector_types::{
    database_definition::DatabaseDefinition,
    transport::{TcpTransport, Transport},
};
use runtime::pg::{PgTransportFactory, PgTransportFactoryError, PgTransportFactoryInner, PgTransportFactoryResult};

pub struct LocalPgTransportFactory {
    transports: Arc<HashMap<String, TcpTransport>>,
}

impl LocalPgTransportFactory {
    pub fn runtime_factory(transports: Arc<HashMap<String, TcpTransport>>) -> PgTransportFactory {
        PgTransportFactory::new(Box::new(LocalPgTransportFactory { transports }))
    }
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for LocalPgTransportFactory {
    async fn try_new(
        &self,
        _name: &str,
        _database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>> {
        unimplemented!("use the cached version")
    }

    fn fetch_cached(&self, name: &str) -> PgTransportFactoryResult<&dyn Transport> {
        let tcp_transport = self
            .transports
            .get(name)
            .ok_or_else(|| PgTransportFactoryError::TransportNotFound(name.to_string()))?;

        Ok(tcp_transport)
    }
}
