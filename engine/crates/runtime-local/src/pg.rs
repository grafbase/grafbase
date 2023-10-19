use postgres_types::{
    database_definition::DatabaseDefinition,
    transport::{TcpTransport, Transport},
};
use runtime::pg::{PgTransportFactory, PgTransportFactoryError, PgTransportFactoryInner, PgTransportFactoryResult};

pub struct LocalPgTransportFactory;

impl LocalPgTransportFactory {
    pub fn runtime_factory() -> PgTransportFactory {
        PgTransportFactory::new(Box::new(LocalPgTransportFactory))
    }
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for LocalPgTransportFactory {
    async fn try_new(
        &self,
        _name: &str,
        database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>> {
        let tcp_transport = TcpTransport::new(database_definition.connection_string())
            .await
            .map_err(PgTransportFactoryError::TransportCreation)?;

        Ok(Box::new(tcp_transport))
    }
}
