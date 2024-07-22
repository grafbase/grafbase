use deadpool::managed::{self, Metrics};
use redis::{aio::MultiplexedConnection, Client, ClientTlsConfig, RedisError, RedisResult, TlsCertificates};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct Manager {
    client: Client,
    ping_number: AtomicUsize,
}

pub(super) struct TlsConfig {
    pub client_cert: Vec<u8>,
    pub client_key: Vec<u8>,
    pub root_cert: Option<Vec<u8>>,
}

impl Manager {
    pub fn new(url: &str, tls: Option<TlsConfig>) -> RedisResult<Self> {
        let client = match tls {
            Some(config) => Client::build_with_tls(
                url,
                TlsCertificates {
                    client_tls: Some(ClientTlsConfig {
                        client_cert: config.client_cert,
                        client_key: config.client_key,
                    }),
                    root_cert: config.root_cert,
                },
            )?,
            None => Client::open(url)?,
        };

        Ok(Self {
            client,
            ping_number: AtomicUsize::new(0),
        })
    }
}

impl managed::Manager for Manager {
    type Type = MultiplexedConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<MultiplexedConnection, Self::Error> {
        let conn = self.client.get_multiplexed_async_connection().await?;

        Ok(conn)
    }

    async fn recycle(&self, conn: &mut MultiplexedConnection, _: &Metrics) -> managed::RecycleResult<Self::Error> {
        let ping_number = self.ping_number.fetch_add(1, Ordering::Relaxed).to_string();

        // Using pipeline to avoid roundtrip for UNWATCH
        let (n,) = redis::Pipeline::with_capacity(2)
            .cmd("UNWATCH")
            .ignore()
            .cmd("PING")
            .arg(&ping_number)
            .query_async::<_, (String,)>(conn)
            .await?;

        if n == ping_number {
            Ok(())
        } else {
            Err(managed::RecycleError::message("Invalid PING response"))
        }
    }
}
