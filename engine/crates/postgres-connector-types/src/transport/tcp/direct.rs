use async_trait::async_trait;
use futures::channel::oneshot;
use futures::stream::BoxStream;
use serde_json::Value;

use crate::error::Error;
use crate::transport::ext::TransportTransactionExt;
use crate::transport::tcp::executor;
use crate::transport::{Transport, TransportTransaction};

pub struct DirectTcpTransport {
    client: tokio_postgres::Client,
    connection_string: String,
    close_recv: oneshot::Receiver<()>,
}

impl DirectTcpTransport {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(connection_string: &str) -> crate::Result<Self> {
        let mut roots = rustls::RootCertStore::empty();

        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
            roots.add(cert).expect("could not add platform cert");
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config);

        let (client, connection) = tokio_postgres::connect(connection_string, tls)
            .await
            .map_err(|error| crate::error::Error::Connection(error.to_string()))?;

        let (close_send, close_recv) = oneshot::channel();

        async_runtime::spawn(async move {
            if let Err(e) = connection.await {
                tracing::trace!("postgres connection error: {e}");
            }

            if close_send.send(()).is_err() {
                tracing::trace!("did not close the postgres connection properly (matters only in API)");
            }
        });

        let this = Self {
            client,
            connection_string: connection_string.to_string(),
            close_recv,
        };

        Ok(this)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(connection_string: &str) -> crate::Result<Self> {
        use std::str::FromStr;

        let url = url::Url::parse(connection_string)
            .map_err(|error| crate::error::Error::InvalidConnectionString(error.to_string()))?;

        let config = tokio_postgres::config::Config::from_str(connection_string)
            .map_err(|error| crate::error::Error::Connection(error.to_string()))?;

        let hostname = url.host_str().ok_or_else(|| {
            crate::error::Error::InvalidConnectionString(String::from(
                "the connection string does not define a valid hostname",
            ))
        })?;

        let socket = worker::Socket::builder()
            .connect(hostname, url.port().unwrap_or(5432))
            .map_err(|error| crate::error::Error::Connection(error.to_string()))?;

        let (client, connection) = config
            .connect_raw(socket, tokio_postgres::tls::NoTls)
            .await
            .map_err(|error| crate::error::Error::Connection(error.to_string()))?;

        let (close_send, close_recv) = oneshot::channel();

        async_runtime::spawn(async move {
            if let Err(e) = connection.await {
                tracing::trace!("postgres connection error: {e}");
            }

            if close_send.send(()).is_err() {
                tracing::trace!("did not close the postgres connection properly (matters only in API)");
            }
        });

        let this = Self {
            client,
            connection_string: connection_string.to_string(),
            close_recv,
        };

        Ok(this)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for DirectTcpTransport {
    async fn close(self) -> crate::Result<()> {
        drop(self.client);
        self.close_recv.await.map_err(|e| Error::Internal(e.to_string()))
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        executor::execute(&self.client, query, params).await
    }

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        executor::query(&self.client, query, params)
    }

    fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TransportTransactionExt for DirectTcpTransport {
    async fn transaction(&mut self) -> crate::Result<TransportTransaction<'_>> {
        self.client
            .transaction()
            .await
            .map(TransportTransaction::Direct)
            .map_err(|e| Error::Transaction(e.to_string()))
    }
}
