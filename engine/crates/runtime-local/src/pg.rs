use std::{collections::HashMap, sync::Arc};

use postgres_connector_types::transport::Transport;
use runtime::pg::{PgTransportFactoryError, PgTransportFactoryInner, PgTransportFactoryResult};

#[derive(Clone)]
pub struct LocalPgTransportFactory {
    transports: Arc<HashMap<String, Arc<dyn Transport>>>,
}

impl LocalPgTransportFactory {
    pub fn new(transports: HashMap<String, Arc<dyn Transport>>) -> Self {
        LocalPgTransportFactory {
            transports: Arc::new(transports),
        }
    }
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for LocalPgTransportFactory {
    async fn try_get(&self, name: &str) -> PgTransportFactoryResult<Arc<dyn Transport>> {
        let transport = self
            .transports
            .get(name)
            .cloned()
            .ok_or_else(|| PgTransportFactoryError::TransportNotFound(name.to_string()))?;

        Ok(transport)
    }
}
