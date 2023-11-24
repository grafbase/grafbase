use postgres_connector_types::{database_definition::DatabaseDefinition, error::Error, transport::Transport};
use runtime::pg::{PgTransportFactory, PgTransportFactoryError, PgTransportFactoryInner, PgTransportFactoryResult};
use std::collections::HashMap;

pub(crate) struct WasmTransport(pub String);

#[async_trait::async_trait]
impl Transport for WasmTransport {
    async fn close(self) -> postgres_connector_types::Result<()> {
        Ok(())
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<serde_json::Value>) -> postgres_connector_types::Result<i64> {
        todo!();
    }

    fn parameterized_query<'a>(
        &'a self,
        query: &'a str,
        params: Vec<serde_json::Value>,
    ) -> BoxStream<'a, Result<serde_json::Value, Error>> {
        todo!();
    }

    fn connection_string(&self) -> &str {
        self.0.as_str()
    }
}

pub(crate) fn make_pg_transport_factory(transports: HashMap<String, WasmTransport>) -> PgTransportFactory {
    let factory_impl = PgTransportFactoryImpl { transports };
    PgTransportFactory::new(Box::new(factory_impl))
}

struct PgTransportFactoryImpl {
    transports: HashMap<String, WasmTransport>,
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for PgTransportFactoryImpl {
    async fn try_new(
        &self,
        name: &str,
        database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>> {
        tracing::error!("got to the factory new");
        panic!()
    }

    fn fetch_cached(&self, name: &str) -> PgTransportFactoryResult<&dyn Transport> {
        tracing::info!("fetching cached transport `{name}`");
        self.transports
            .get(name)
            .ok_or_else(|| Err(PgTransportFactoryError::TransportNotFound(name.to_owned())))
    }
}
