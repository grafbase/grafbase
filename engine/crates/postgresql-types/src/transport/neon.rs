use super::{ExecuteResponse, QueryResponse, Transport};
use crate::error::Error;
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    ClientBuilder, Url,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

const LOCAL_TEST_HOSTNAME: &str = "db.localtest.me";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct NeonRequest<'a> {
    query: &'a str,
    params: Vec<Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    name: String,
    /// The index of the column in the row
    #[serde(rename = "columnID")]
    column_id: i16,
    /// The type ID the column
    #[serde(rename = "dataTypeID")]
    data_type_id: u32,
}

impl From<Field> for super::Column {
    fn from(field: Field) -> Self {
        Self {
            name: field.name,
            column_id: field.column_id,
            data_type_id: field.data_type_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NeonResponse<T> {
    fields: Vec<Field>,
    row_count: Option<usize>,
    rows: Vec<T>,
}

pub struct NeonTransport {
    http_client: reqwest::Client,
    http_url: String,
    connection_string: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeonError {
    code: String,
    message: String,
}

impl NeonTransport {
    pub fn new(ray_id: &str, connection_string: &str) -> crate::Result<Self> {
        let url = Url::parse(connection_string)?;

        let http_host = match url.host_str() {
            Some(host_str) => host_str,
            None => LOCAL_TEST_HOSTNAME,
        };

        let http_port = if http_host == LOCAL_TEST_HOSTNAME { 4444 } else { 443 };
        let http_url = format!("https://{http_host}:{http_port}/sql");

        let mut headers = HeaderMap::new();

        headers.insert(
            "Neon-Connection-String",
            HeaderValue::from_str(connection_string)
                .map_err(|_| Error::InvalidConnectionString("the URL must only use ASCII characeters".to_string()))?,
        );
        headers.insert(
            "x-grafbase-fetch-trace-id",
            HeaderValue::from_str(ray_id).expect("must be valid"),
        );
        headers.insert("Neon-Raw-Text-Output", HeaderValue::from_static("false"));
        headers.insert("Neon-Array-Mode", HeaderValue::from_static("false"));
        headers.insert("Neon-Pool-Opt-In", HeaderValue::from_static("false"));

        let http_client = Self::client_builder(http_host, headers).build()?;

        Ok(Self {
            http_client,
            http_url,
            connection_string: connection_string.to_string(),
        })
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }

    #[cfg(target_arch = "wasm32")]
    fn client_builder(_: &str, headers: HeaderMap) -> ClientBuilder {
        reqwest::ClientBuilder::new().default_headers(headers)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn client_builder(http_host: &str, headers: HeaderMap) -> ClientBuilder {
        let builder = reqwest::ClientBuilder::new().default_headers(headers);

        if http_host == LOCAL_TEST_HOSTNAME {
            builder.danger_accept_invalid_certs(true)
        } else {
            builder
        }
    }

    async fn request<T>(&self, request: &NeonRequest<'_>) -> crate::Result<NeonResponse<T>>
    where
        T: DeserializeOwned + Send,
    {
        let response = self.http_client.post(&self.http_url).json(request).send().await?;

        if response.status().is_client_error() {
            let error: NeonError = response.json().await?;

            Err(Error::Query {
                code: error.code,
                message: error.message,
            })
        } else {
            Ok(response.error_for_status()?.json().await?)
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for NeonTransport {
    async fn parameterized_query<T>(&self, query: &str, params: Vec<Value>) -> crate::Result<QueryResponse<T>>
    where
        T: DeserializeOwned + Send,
    {
        let response = self.request(&NeonRequest { query, params }).await?;

        Ok(QueryResponse {
            columns: response.fields.into_iter().map(super::Column::from).collect(),
            row_count: response.row_count.unwrap_or(0),
            rows: response.rows,
        })
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<ExecuteResponse> {
        let response = self.request::<Value>(&NeonRequest { query, params }).await?;

        Ok(ExecuteResponse {
            row_count: response.row_count.unwrap_or(0),
        })
    }

    fn connection_string(&self) -> &str {
        &self.connection_string
    }
}
