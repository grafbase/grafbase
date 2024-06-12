use std::time::Duration;

use async_trait::async_trait;
use deadpool_postgres::{Object, Pool, Runtime, Timeouts};
use futures::stream::BoxStream;
use futures::StreamExt as _;
use serde_json::Value;
use tokio_postgres::GenericClient;

use crate::error::Error;
use crate::transport::ext::TransportTransactionExt;
use crate::transport::tcp::executor;
use crate::transport::{Transport, TransportTransaction};

#[derive(Default)]
pub struct PoolingConfig {
    pub max_size: Option<usize>,
    pub wait_timeout: Option<Duration>,
    pub create_timeout: Option<Duration>,
    pub recycle_timeout: Option<Duration>,
}
pub struct PooledTcpTransport {
    pool: Pool,
    connection_string: String,
}
impl PooledTcpTransport {
    pub async fn new(connection_string: &str, pool_config: PoolingConfig) -> crate::Result<Self> {
        let mut roots = rustls::RootCertStore::empty();

        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
            roots.add(cert).expect("could not add platform cert");
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config);

        let mut config = deadpool_postgres::Config::new();
        config.url = Some(connection_string.to_string());

        let mut pool_builder = config.builder(tls)?.runtime(Runtime::Tokio1).timeouts(Timeouts {
            wait: pool_config.wait_timeout,
            create: pool_config.create_timeout,
            recycle: pool_config.recycle_timeout,
        });

        if let Some(max_size) = pool_config.max_size {
            pool_builder = pool_builder.max_size(max_size);
        }

        let pool = pool_builder.build()?;

        Ok(Self {
            connection_string: connection_string.to_string(),
            pool,
        })
    }

    pub async fn connection(&self) -> crate::Result<PooledTcpConnection> {
        let connection = self.pool.get().await.map_err(|e| Error::Deadpool(e.to_string()))?;
        Ok(PooledTcpConnection {
            connection,
            connection_string: self.connection_string.clone(),
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for PooledTcpTransport {
    async fn close(self) -> crate::Result<()> {
        Ok(())
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        self.connection().await?.parameterized_execute(query, params).await
    }

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        Box::pin(async_stream::try_stream! {
            let connection = self.connection().await?;
            let mut stream = connection.parameterized_query(query, params);

            while let Some(Ok(row)) = stream.next().await {
                yield row;
            }
        })
    }

    fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

pub struct PooledTcpConnection {
    connection: Object,
    connection_string: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for PooledTcpConnection {
    async fn close(self) -> crate::Result<()> {
        Ok(())
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        executor::execute(self.connection.client(), query, params).await
    }

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        executor::query(self.connection.client(), query, params)
    }

    fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TransportTransactionExt for PooledTcpConnection {
    async fn transaction(&mut self) -> crate::Result<TransportTransaction<'_>> {
        self.connection
            .transaction()
            .await
            .map(TransportTransaction::Pooled)
            .map_err(|e| Error::Transaction(e.to_string()))
    }
}
