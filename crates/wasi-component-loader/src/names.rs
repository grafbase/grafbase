// Shared

pub(crate) const CONTEXT_RESOURCE: &str = "context";
pub(crate) const CONTEXT_SET_METHOD: &str = "[method]context.set";
pub(crate) const CONTEXT_GET_METHOD: &str = "[method]context.get";
pub(crate) const CONTEXT_DELETE_METHOD: &str = "[method]context.delete";

pub(crate) const SHARED_CONTEXT_RESOURCE: &str = "shared-context";
pub(crate) const SHARED_CONTEXT_GET_METHOD: &str = "[method]shared-context.get";
pub(crate) const SHARED_CONTEXT_TRACE_ID_METHOD: &str = "[method]shared-context.trace-id";

pub(crate) const HTTP_CLIENT_RESOURCE: &str = "http-client";
pub(crate) const HTTP_CLIENT_EXECUTE_FUNCTION: &str = "[static]http-client.execute";
pub(crate) const HTTP_CLIENT_EXECUTE_MANY_FUNCTION: &str = "[static]http-client.execute-many";

pub(crate) const ACCESS_LOG_RESOURCE: &str = "access-log";
pub(crate) const ACCESS_LOG_SEND_FUNCTION: &str = "[static]access-log.send";

// Hooks

pub(crate) const INIT_HOOKS_FUNCTION: &str = "init-hooks";
pub(crate) const GATEWAY_HOOK_FUNCTION: &str = "on-gateway-request";
pub(crate) const AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-pre-execution";
pub(crate) const AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION: &str = "authorize-node-pre-execution";
pub(crate) const AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-parent-edge-post-execution";
pub(crate) const AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-node-post-execution";
pub(crate) const AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-post-execution";
pub(crate) const ON_SUBGRAGH_REQUEST_HOOK_FUNCTION: &str = "on-subgraph-request";

pub(crate) const ON_SUBGRAPH_RESPONSE_FUNCTION: &str = "on-subgraph-response";
pub(crate) const ON_OPERATION_RESPONSE_FUNCTION: &str = "on-operation-response";
pub(crate) const ON_HTTP_RESPONSE_FUNCTION: &str = "on-http-response";

pub(crate) const HEADERS_RESOURCE: &str = "headers";
pub(crate) const HEADERS_SET_METHOD: &str = "[method]headers.set";
pub(crate) const HEADERS_GET_METHOD: &str = "[method]headers.get";
pub(crate) const HEADERS_DELETE_METHOD: &str = "[method]headers.delete";
pub(crate) const HEADERS_ENTRIES_METHOD: &str = "[method]headers.entries";

pub(crate) const SUBGRAPH_REQUEST_RESOURCE: &str = "subgraph-request";
pub(crate) const SUBGRAPH_REQUEST_GET_METHOD_METHOD: &str = "[method]subgraph-request.method";
pub(crate) const SUBGRAPH_REQUEST_SET_METHOD_METHOD: &str = "[method]subgraph-request.set-method";
pub(crate) const SUBGRAPH_REQUEST_GET_URL_METHOD: &str = "[method]subgraph-request.url";
pub(crate) const SUBGRAPH_REQUEST_SET_URL_METHOD: &str = "[method]subgraph-request.set-url";
pub(crate) const SUBGRAPH_REQUEST_GET_HEADERS_METHOD: &str = "[method]subgraph-request.headers";
