use std::{collections::HashMap, pin::Pin, sync::LazyLock};

use axum::{Router, body::Body};
use futures::{Stream, StreamExt};
use http::{
    Method, Request,
    header::{ACCEPT, CONTENT_TYPE},
};
use sse_stream::{Sse, SseStream};
use tower::Service;

const BASE_URL: &str = "http://127.0.0.1";

pub struct McpStream {
    router: Router,
    command_uri: String,
    stream: Pin<Box<dyn Stream<Item = Result<Sse, sse_stream::Error>> + Send>>,
    id: usize,
    server_info: Option<InitializeResponse>,
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
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    pub instructions: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub tools: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpResponse<T> {
    result: Option<T>,
    error: Option<McpError>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolResponse {
    content: Vec<ToolContent>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolContent {
    text: String,
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

        this.server_info = Some(serde_json::from_value(this.initialize().await.unwrap()).unwrap());
        this.send_notification("notifications/initialized").await;

        this
    }

    async fn initialize(&mut self) -> Result<serde_json::Value, McpError> {
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
        let response: McpResponse<serde_json::Value> = serde_json::from_str(&sse.data.unwrap()).unwrap();

        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (_, Some(error)) => Err(error),
            _ => unreachable!(),
        }
    }

    pub fn server_info(&self) -> InitializeResponse {
        self.server_info.clone().unwrap()
    }

    pub async fn list_tools(&mut self) -> Result<serde_json::Value, McpError> {
        self.send_command("tools/list", EMPTY_PARAMS.clone()).await;

        let sse = self.fetch_response().await;
        let response: McpResponse<serde_json::Value> = serde_json::from_str(&sse.data.unwrap()).unwrap();

        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (_, Some(error)) => Err(error),
            _ => unreachable!(),
        }
    }

    pub async fn call_tool(
        &mut self,
        name: &'static str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        self.send_command("tools/call", ToolsCallParams { name, arguments })
            .await;

        let sse = self.fetch_response().await;
        let response: McpResponse<ToolResponse> = serde_json::from_str(&sse.data.unwrap()).unwrap();

        match (response.result, response.error) {
            (Some(mut result), _) => {
                let result = serde_json::from_str(&result.content.pop().unwrap().text).unwrap();

                Ok(result)
            }
            (_, Some(error)) => Err(error),
            _ => unreachable!(),
        }
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
