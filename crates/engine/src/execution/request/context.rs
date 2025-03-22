use grafbase_telemetry::grafbase_client::Client;
use runtime::authentication::LegacyToken;

use crate::{Runtime, engine::WasmContext, graphql_over_http::ResponseFormat};

/// Context only used early in the request processing before generating the RequestContext used
/// everywhere else. Contrary to the RequestContext this one never fails to be created.
pub(crate) struct EarlyHttpContext {
    pub method: http::method::Method,
    pub uri: http::Uri,
    pub response_format: ResponseFormat,
    pub include_grafbase_response_extension: bool,
}

/// Context associated with the HTTP request. For batch requests and a websocket session, a single RequestContext is
/// created and shared.
pub(crate) struct RequestContext {
    pub mutations_allowed: bool,
    pub headers: http::HeaderMap,
    pub websocket_init_payload: Option<serde_json::Map<String, serde_json::Value>>,
    pub response_format: ResponseFormat,
    pub client: Option<Client>,
    pub token: LegacyToken,
    pub subgraph_default_headers: http::HeaderMap,
    pub include_grafbase_response_extension: bool,
}

/// Context associated with a single operation within an HTTP request.
/// Every single operation, whether in a websocket session or batch request will have its own
/// GraphqlContext.
pub(crate) struct GraphqlRequestContext<R: Runtime> {
    pub wasm_context: WasmContext<R>,
    pub subgraph_default_headers_override: Option<http::HeaderMap>,
}
