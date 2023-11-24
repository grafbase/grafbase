use postgres_connector_types::{database_definition::DatabaseDefinition, transport::Transport};
use runtime::pg::{PgTransportFactory, PgTransportFactoryInner, PgTransportFactoryResult};

pub(crate) fn make_pg_transport_factory() -> PgTransportFactory {
    let factory_impl = PgTransportFactoryImpl;
    PgTransportFactory::new(Box::new(factory_impl))
}

struct PgTransportFactoryImpl;

#[async_trait::async_trait]
impl PgTransportFactoryInner for PgTransportFactoryImpl {
    async fn try_new(
        &self,
        name: &str,
        database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>> {
        tracing::warn!("got to the factory new");
        todo!()
    }

    fn fetch_cached(&self, name: &str) -> PgTransportFactoryResult<&dyn Transport> {
        tracing::warn!("got to the factory fetch cached");
        todo!()
    }
}
