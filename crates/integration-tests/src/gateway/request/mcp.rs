use axum::{Router, body::Body};
use futures::{Stream, StreamExt};
use http::{
    Method, Request,
    header::{ACCEPT, CONTENT_TYPE},
};
use rmcp::{
    ServiceExt,
    model::{ProtocolVersion, ServerJsonRpcMessage},
    transport::{
        StreamableHttpClientTransport,
        common::{
            client_side_sse::ExponentialBackoff,
            http_header::{EVENT_STREAM_MIME_TYPE, HEADER_LAST_EVENT_ID, HEADER_SESSION_ID, JSON_MIME_TYPE},
        },
        streamable_http_client::{
            StreamableHttpClientTransportConfig, StreamableHttpError, StreamableHttpPostResponse,
        },
    },
};
use rmcp::{model::ServerCapabilities, transport::streamable_http_client::StreamableHttpClient};
use sse_stream::{Sse, SseStream};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, LazyLock},
};
use tower::Service;

const BASE_URL: &str = "http://127.0.0.1";

pub struct McpHttpClient {
    client: rmcp::service::RunningService<rmcp::service::RoleClient, rmcp::model::InitializeRequestParam>,
}

pub struct McpHttpClientBuilder {
    pub(crate) router: Router,
    pub(crate) path: String,
    pub(crate) headers: http::HeaderMap,
}

impl McpHttpClientBuilder {
    pub fn new(router: Router, path: impl Into<String>) -> Self {
        Self {
            router,
            path: path.into(),
            headers: Default::default(),
        }
    }

    pub fn with_headers(mut self, headers: http::HeaderMap) -> Self {
        self.headers = headers;
        self
    }
}

impl std::future::IntoFuture for McpHttpClientBuilder {
    type Output = McpHttpClient;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { McpHttpClient::new(self.router, &self.path, self.headers).await })
    }
}

#[derive(Debug, Clone)]
struct RouterClient(Router, http::HeaderMap);

impl StreamableHttpClient for RouterClient {
    type Error = std::io::Error;

    async fn post_message(
        &self,
        uri: std::sync::Arc<str>,
        message: rmcp::model::ClientJsonRpcMessage,
        session_id: Option<std::sync::Arc<str>>,
        auth_header: Option<String>,
    ) -> Result<
        rmcp::transport::streamable_http_client::StreamableHttpPostResponse,
        rmcp::transport::streamable_http_client::StreamableHttpError<Self::Error>,
    > {
        let mut request_builder = http::Request::builder()
            .method(Method::POST)
            .uri(uri.as_ref())
            .header(ACCEPT, [EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE].join(", "))
            .header(CONTENT_TYPE, JSON_MIME_TYPE);

        for (name, value) in &self.1 {
            request_builder = request_builder.header(name, value);
        }

        if let Some(auth_header) = auth_header {
            request_builder = request_builder.header("Authorization", format!("Bearer {auth_header}"));
        }
        if let Some(session_id) = session_id {
            request_builder = request_builder.header(HEADER_SESSION_ID, session_id.as_ref());
        }

        let request = request_builder
            .body(Body::from(serde_json::to_vec(&message).unwrap()))
            .unwrap();

        let mut router = self.0.clone();
        let Ok(response) = router.as_service().call(request).await;

        if !response.status().is_success() {
            return Err(StreamableHttpError::Client(std::io::Error::other(format!(
                "Non-200 response from MCP: {}",
                response.status()
            ))));
        }

        if response.status() == reqwest::StatusCode::ACCEPTED {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        let content_type = response.headers().get(reqwest::header::CONTENT_TYPE);
        let session_id = response.headers().get(HEADER_SESSION_ID);
        let session_id = session_id.and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        match content_type {
            Some(ct) if ct.as_bytes().starts_with(EVENT_STREAM_MIME_TYPE.as_bytes()) => {
                let event_stream = SseStream::from_byte_stream(response.into_body().into_data_stream()).boxed();
                Ok(StreamableHttpPostResponse::Sse(event_stream, session_id))
            }
            Some(ct) if ct.as_bytes().starts_with(JSON_MIME_TYPE.as_bytes()) => {
                let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
                let message: ServerJsonRpcMessage = serde_json::from_slice(body.as_ref()).unwrap();
                Ok(StreamableHttpPostResponse::Json(message, session_id))
            }
            _ => {
                // unexpected content type
                tracing::error!("unexpected content type: {:?}", content_type);
                Err(StreamableHttpError::UnexpectedContentType(
                    content_type.map(|ct| String::from_utf8_lossy(ct.as_bytes()).to_string()),
                ))
            }
        }
    }

    async fn delete_session(
        &self,
        uri: std::sync::Arc<str>,
        session_id: std::sync::Arc<str>,
        auth_header: Option<String>,
    ) -> Result<(), rmcp::transport::streamable_http_client::StreamableHttpError<Self::Error>> {
        let mut request_builder = http::Request::builder().method(Method::DELETE).uri(uri.as_ref());
        if let Some(auth_header) = auth_header {
            request_builder = request_builder.header("Authorization", format!("Bearer {auth_header}"));
        }
        let request = request_builder
            .header(HEADER_SESSION_ID, session_id.as_ref())
            .body(Body::empty())
            .unwrap();

        let mut router = self.0.clone();
        let Ok(response) = router.as_service().call(request).await;

        if response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            tracing::debug!("this server doesn't support deleting session");
            return Ok(());
        }

        if !response.status().is_success() {
            panic!("Non-200 response from MCP: {}", response.status())
        }

        Ok(())
    }

    async fn get_stream(
        &self,
        uri: std::sync::Arc<str>,
        session_id: std::sync::Arc<str>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
    ) -> Result<
        futures::stream::BoxStream<'static, Result<Sse, sse_stream::Error>>,
        rmcp::transport::streamable_http_client::StreamableHttpError<Self::Error>,
    > {
        let mut request_builder = http::Request::builder()
            .method(http::Method::GET)
            .uri(uri.as_ref())
            .header(ACCEPT, EVENT_STREAM_MIME_TYPE)
            .header(HEADER_SESSION_ID, session_id.as_ref());
        if let Some(last_event_id) = last_event_id {
            request_builder = request_builder.header(HEADER_LAST_EVENT_ID, last_event_id);
        }
        if let Some(auth_header) = auth_header {
            request_builder = request_builder.header("Authorization", format!("Bearer {auth_header}"));
        }
        let request = request_builder.body(Body::empty()).unwrap();
        let mut router = self.0.clone();
        let Ok(response) = router.as_service().call(request).await;
        if response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            return Err(StreamableHttpError::SeverDoesNotSupportSse);
        }

        if !response.status().is_success() {
            panic!("non-200 response from mcp: {}", response.status());
        }

        match response.headers().get(reqwest::header::CONTENT_TYPE) {
            Some(ct) => {
                if !ct.as_bytes().starts_with(EVENT_STREAM_MIME_TYPE.as_bytes()) {
                    return Err(StreamableHttpError::UnexpectedContentType(Some(
                        String::from_utf8_lossy(ct.as_bytes()).to_string(),
                    )));
                }
            }
            None => {
                return Err(StreamableHttpError::UnexpectedContentType(None));
            }
        }
        let event_stream = SseStream::from_byte_stream(response.into_body().into_data_stream()).boxed();
        Ok(event_stream)
    }
}

impl McpHttpClient {
    pub(crate) async fn new(router: Router, path: &str, headers: http::HeaderMap) -> Self {
        let transport = StreamableHttpClientTransport::with_client(
            RouterClient(router, headers),
            StreamableHttpClientTransportConfig {
                uri: Arc::from(format!("http://127.0.0.1{path}").into_boxed_str()),
                retry_config: Arc::new(ExponentialBackoff::default()),
                channel_buffer_capacity: 4096 * 10,
                allow_stateless: true,
            },
        );
        let client_info = rmcp::model::ClientInfo {
            protocol_version: rmcp::model::ProtocolVersion::V_2025_03_26,
            capabilities: rmcp::model::ClientCapabilities::default(),
            client_info: rmcp::model::Implementation {
                name: "grafbase-test-client".to_owned(),
                version: "1.0.0".to_owned(),
            },
        };

        let client = client_info.serve(transport).await.unwrap();

        McpHttpClient { client }
    }

    pub fn server_info(&self) -> McpResponse<InitializeResponse> {
        let response = self.client.peer_info().unwrap().clone();
        McpResponse::Result {
            result: InitializeResponse {
                capabilities: response.capabilities,
                instructions: response.instructions,
                protocol_version: response.protocol_version,
                server_info: response.server_info,
            },
        }
    }

    pub async fn list_tools(&mut self) -> McpResponse<rmcp::model::ListToolsResult> {
        let result = self.client.list_tools(None).await.unwrap();
        McpResponse::Result { result }
    }

    pub async fn call_tool(&mut self, name: &'static str, arguments: serde_json::Value) -> McpResponse<ToolResponse> {
        let result = self
            .client
            .call_tool(rmcp::model::CallToolRequestParam {
                name: name.into(),
                arguments: match arguments {
                    serde_json::Value::Object(map) => Some(map),
                    _ => panic!("bad arguments to call_tool"),
                },
            })
            .await
            .unwrap();

        McpResponse::Result {
            result: ToolResponse {
                content: result
                    .content
                    .into_iter()
                    .map(|item| match item.raw {
                        rmcp::model::RawContent::Text(raw_text_content) => serde_json::from_str(&raw_text_content.text)
                            .map(Content::Json)
                            .unwrap_or(Content::Text(raw_text_content.text)),
                        _ => unreachable!("Non-text tool response"),
                    })
                    .collect(),
                is_error: result.is_error,
            },
        }
    }
}

pub struct McpStream {
    router: Router,
    command_uri: String,
    stream: Pin<Box<dyn Stream<Item = Result<Sse, sse_stream::Error>> + Send>>,
    id: usize,
    server_info: Option<McpResponse<InitializeResponse>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpEvent<T> {
    id: usize,
    method: &'static str,
    params: T,
    jsonrpc: &'static str,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpNotification {
    method: &'static str,
    jsonrpc: &'static str,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Initialize {
    protocol_version: &'static str,
    capabilities: Capabilities,
    client_info: ClientInfo,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ServerCapabilities,
    pub server_info: rmcp::model::Implementation,
    pub instructions: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Capabilities {
    sampling: HashMap<u8, u8>,
    roots: Roots,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Roots {
    list_changed: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientInfo {
    name: &'static str,
    version: &'static str,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum McpResponse<T> {
    Result { result: T },
    Error { error: McpError },
}

impl<T: std::fmt::Display> std::fmt::Display for McpResponse<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpResponse::Result { result } => write!(f, "{result}"),
            McpResponse::Error { error } => write!(f, "{}", serde_json::to_string_pretty(error).unwrap()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ToolResponse {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl std::fmt::Display for ToolResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_error.unwrap_or_default() {
            writeln!(f, "is_error: true")?;
        }
        for (i, content) in self.content.iter().enumerate() {
            if i > 0 {
                writeln!(f, "\n{}\n", "=".repeat(80))?;
            }
            match content {
                Content::Text(text) => write!(f, "{text}")?,
                Content::Json(json) => write!(f, "{}", serde_json::to_string_pretty(json).unwrap())?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Json(serde_json::Value),
}

impl<'de> serde::Deserialize<'de> for ToolResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            content: Vec<ToolContent>,
            is_error: Option<bool>,
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ToolContent {
            text: String,
        }

        let raw = Raw::deserialize(deserializer)?;
        Ok(ToolResponse {
            content: raw
                .content
                .into_iter()
                .map(|c| match serde_json::from_str(&c.text) {
                    Ok(json) => Ok(Content::Json(json)),
                    Err(_) => Ok(Content::Text(c.text)),
                })
                .collect::<Result<_, _>>()?,
            is_error: raw.is_error,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCallParams {
    name: &'static str,
    arguments: serde_json::Value,
}

static EMPTY_PARAMS: LazyLock<HashMap<u8, u8>> = LazyLock::new(HashMap::new);

impl McpStream {
    pub async fn new(mut router: Router, path: &str) -> Self {
        let uri = format!("{BASE_URL}{path}");

        let req = Request::builder()
            .uri(&uri)
            .header(ACCEPT, "text/event-stream")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.call(req).await.unwrap();
        let body_stream = response.into_body().into_data_stream();

        let mut stream = SseStream::from_byte_stream(body_stream).boxed();
        let msg = stream.next().await.unwrap().unwrap();

        assert_eq!(Some("endpoint"), msg.event.as_deref());

        let path = msg.data.unwrap();
        let command_uri = format!("{BASE_URL}{path}");

        let mut this = Self {
            id: 0,
            router,
            command_uri,
            stream,
            server_info: None,
        };

        this.server_info = Some(this.initialize().await);
        this.send_notification("notifications/initialized").await;

        this
    }

    async fn initialize(&mut self) -> McpResponse<InitializeResponse> {
        let init = Initialize {
            protocol_version: "2024-11-05",
            capabilities: Capabilities {
                sampling: HashMap::new(),
                roots: Roots { list_changed: true },
            },
            client_info: ClientInfo {
                name: "grafbase-integration-tests",
                version: "4.2.0",
            },
        };

        self.send_command("initialize", init).await;

        let sse = self.fetch_response().await;
        let data = sse.data.unwrap();
        serde_json::from_str(&data).unwrap_or_else(|_| panic!("Failed to parse tool list response: {data}"))
    }

    pub fn server_info(&self) -> McpResponse<InitializeResponse> {
        self.server_info.clone().unwrap()
    }

    pub async fn list_tools(&mut self) -> McpResponse<serde_json::Value> {
        self.send_command("tools/list", EMPTY_PARAMS.clone()).await;

        let sse = self.fetch_response().await;
        let data = sse.data.unwrap();
        serde_json::from_str(&data).unwrap_or_else(|_| panic!("Failed to parse tool list response: {data}"))
    }

    pub async fn call_tool(&mut self, name: &'static str, arguments: serde_json::Value) -> McpResponse<ToolResponse> {
        self.send_command("tools/call", ToolsCallParams { name, arguments })
            .await;

        let sse = self.fetch_response().await;
        let data = sse.data.unwrap();
        serde_json::from_str(&data).unwrap_or_else(|_| panic!("Failed to parse tool list response: {data}"))
    }

    pub async fn send_command<S>(&mut self, method: &'static str, msg: S)
    where
        S: serde::Serialize + std::fmt::Debug,
    {
        let event = McpEvent {
            id: self.id,
            method,
            params: msg,
            jsonrpc: "2.0",
        };

        self.id += 1;

        let request = Request::builder()
            .method(Method::POST)
            .uri(&self.command_uri)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(Body::from(serde_json::to_vec(&event).unwrap()))
            .unwrap();

        self.router.call(request).await.unwrap();
    }

    pub async fn send_notification(&mut self, method: &'static str) {
        let notification = McpNotification { method, jsonrpc: "2.0" };

        let request = Request::builder()
            .method(Method::POST)
            .uri(&self.command_uri)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&notification).unwrap()))
            .unwrap();

        self.router.call(request).await.unwrap();
    }

    pub async fn fetch_response(&mut self) -> Sse {
        self.stream.next().await.unwrap().unwrap()
    }
}
