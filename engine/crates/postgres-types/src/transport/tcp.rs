mod conversion;

use self::conversion::json_to_string;

use super::Transport;
use crate::error::Error;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::{pin_mut, StreamExt};
use serde_json::Value;

pub struct TcpTransport {
    client: tokio_postgres::Client,
    connection_string: String,
}

impl TcpTransport {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(connection_string: &str) -> crate::Result<Self> {
        let mut roots = rustls::RootCertStore::empty();

        for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
            roots
                .add(&rustls::Certificate(cert.0))
                .expect("could not add platform cert");
        }

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config);

        let (client, connection) = tokio_postgres::connect(connection_string, tls)
            .await
            .map_err(|error| crate::error::Error::Connection(error.to_string()))?;

        async_runtime::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {e}");
            }
        });

        let this = Self {
            client,
            connection_string: connection_string.to_string(),
        };

        Ok(this)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(connection_string: &str) -> crate::Result<Self> {
        let url = url::Url::parse(connection_string)
            .map_err(|error| crate::error::Error::InvalidConnectionString(error.to_string()))?;

        let username = percent_encoding::percent_decode_str(url.username())
            .decode_utf8()
            .unwrap_or_default();

        let password = percent_encoding::percent_decode_str(url.password().unwrap_or_default())
            .decode_utf8()
            .unwrap_or_default();

        let dbname = match url.path_segments() {
            Some(mut segments) => segments.next().unwrap_or("postgres"),
            None => "postgres",
        };

        let mut config = tokio_postgres::config::Config::new();
        config.user(&username);
        config.password(password.as_ref());
        config.dbname(dbname);

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

        async_runtime::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {e}");
            }
        });

        let this = Self {
            client,
            connection_string: connection_string.to_string(),
        };

        Ok(this)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for TcpTransport {
    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        Box::pin(async_stream::try_stream! {
            let params = json_to_string(params);
            let row_stream = self.client.query_raw_txt(query, params).await?;

            pin_mut!(row_stream);

            while let Some(row) = row_stream.next().await {
                yield serde_json::from_value(conversion::row_to_json(&row?)?)?;
            }
        })
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        let params = json_to_string(params);
        let row_stream = self.client.query_raw_txt(query, params).await?;

        pin_mut!(row_stream);

        while (row_stream.next().await).is_some() {}

        let command_tag = row_stream.command_tag().unwrap_or_default();
        let mut command_tag_split = command_tag.split(' ');
        let command_tag_name = command_tag_split.next().unwrap_or_default();

        let row_count = if command_tag_name == "INSERT" {
            // INSERT returns OID first and then number of rows
            command_tag_split.nth(1)
        } else {
            // other commands return number of rows (if any)
            command_tag_split.next()
        }
        .and_then(|s| s.parse::<i64>().ok());

        Ok(row_count.unwrap_or_default())
    }

    fn connection_string(&self) -> &str {
        &self.connection_string
    }
}
