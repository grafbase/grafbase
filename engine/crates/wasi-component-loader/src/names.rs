pub(crate) static COMPONENT_TYPES: &str = "component:grafbase/types";
pub(crate) static GATEWAY_REQUEST_INTERFACE: &str = "component:grafbase/gateway-request";
pub(crate) static AUTHORIZATION_INTERFACE: &str = "component:grafbase/authorization";
pub(crate) static SUBGRAPH_REQUEST_INTERFACE: &str = "component:grafbase/subgraph-request";
pub(crate) static RESPONSES_INTERFACE: &str = "component:grafbase/responses";

pub(crate) static GATEWAY_HOOK_FUNCTION: &str = "on-gateway-request";
pub(crate) static AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-pre-execution";
pub(crate) static AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION: &str = "authorize-node-pre-execution";
pub(crate) static AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-parent-edge-post-execution";
pub(crate) static AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-node-post-execution";
pub(crate) static AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION: &str = "authorize-edge-post-execution";
pub(crate) static ON_SUBGRAGH_REQUEST_HOOK_FUNCTION: &str = "on-subgraph-request";

pub(crate) static ON_SUBGRAPH_RESPONSE_FUNCTION: &str = "on-subgraph-response";
pub(crate) static ON_OPERATION_RESPONSE_FUNCTION: &str = "on-operation-response";
pub(crate) static ON_HTTP_RESPONSE_FUNCTION: &str = "on-http-response";

pub(crate) static HEADERS_RESOURCE: &str = "headers";
pub(crate) static HEADERS_SET_METHOD: &str = "[method]headers.set";
pub(crate) static HEADERS_GET_METHOD: &str = "[method]headers.get";
pub(crate) static HEADERS_DELETE_METHOD: &str = "[method]headers.delete";
pub(crate) static HEADERS_ENTRIES_METHOD: &str = "[method]headers.entries";

pub(crate) static CONTEXT_RESOURCE: &str = "context";
pub(crate) static CONTEXT_SET_METHOD: &str = "[method]context.set";
pub(crate) static CONTEXT_GET_METHOD: &str = "[method]context.get";
pub(crate) static CONTEXT_DELETE_METHOD: &str = "[method]context.delete";

pub(crate) static SHARED_CONTEXT_RESOURCE: &str = "shared-context";
pub(crate) static SHARED_CONTEXT_GET_METHOD: &str = "[method]shared-context.get";
pub(crate) static SHARED_CONTEXT_ACCESS_LOG_METHOD: &str = "[method]shared-context.log-access";
pub(crate) static SHARED_CONTEXT_TRACE_ID_METHOD: &str = "[method]shared-context.trace-id";
