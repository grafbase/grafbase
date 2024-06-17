use std::{collections::HashMap, sync::Arc};

use futures_util::{future::BoxFuture, Future};
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

#[derive(Clone)]
pub struct LazyPgConnectionsPool {
    transports_by_connection_string: Arc<tokio::sync::Mutex<HashMap<String, Arc<dyn Transport>>>>,
    builder: Arc<dyn Fn(String) -> BoxFuture<'static, Arc<dyn Transport>> + Send + Sync>,
}

impl LazyPgConnectionsPool {
    pub fn new<F: Future<Output = T> + Send + 'static, T: Transport + 'static>(
        builder: impl Fn(String) -> F + 'static + Send + Sync,
    ) -> Self {
        Self {
            transports_by_connection_string: Default::default(),
            builder: Arc::new(move |connection_string: String| {
                let fut = builder(connection_string);
                Box::pin(async {
                    let transport = fut.await;
                    Arc::new(transport) as Arc<dyn Transport>
                })
            }),
        }
    }

    pub async fn get(&self, connection_string: &str) -> Arc<dyn Transport> {
        let mut transports = self.transports_by_connection_string.lock().await;
        if let Some(transport) = transports.get(connection_string) {
            return transport.clone();
        }
        let transport = (self.builder)(connection_string.to_string()).await;
        transports.insert(connection_string.to_string(), transport.clone());
        transport
    }

    pub fn to_transport_factory(&self, name_to_connection_string: HashMap<String, String>) -> LazyPgTransportFactory {
        LazyPgTransportFactory {
            pool: self.clone(),
            name_to_connection_string,
        }
    }
}

pub struct LazyPgTransportFactory {
    pool: LazyPgConnectionsPool,
    name_to_connection_string: HashMap<String, String>,
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for LazyPgTransportFactory {
    async fn try_get(&self, name: &str) -> PgTransportFactoryResult<Arc<dyn Transport>> {
        let connection_string = self.name_to_connection_string.get(name).unwrap();
        Ok(self.pool.get(connection_string).await)
    }
}
