pub(crate) const COMPONENT_TYPES: &str = "component:grafbase/types";
pub(crate) const GATEWAY_REQUEST_INTERFACE: &str = "component:grafbase/gateway-request";
pub(crate) const AUTHORIZATION_INTERFACE: &str = "component:grafbase/authorization";
pub(crate) const SUBGRAPH_REQUEST_INTERFACE: &str = "component:grafbase/subgraph-request";
pub(crate) const RESPONSES_INTERFACE: &str = "component:grafbase/responses";

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

pub(crate) const CONTEXT_RESOURCE: &str = "context";
pub(crate) const CONTEXT_SET_METHOD: &str = "[method]context.set";
pub(crate) const CONTEXT_GET_METHOD: &str = "[method]context.get";
pub(crate) const CONTEXT_DELETE_METHOD: &str = "[method]context.delete";

pub(crate) const SHARED_CONTEXT_RESOURCE: &str = "shared-context";
pub(crate) const SHARED_CONTEXT_GET_METHOD: &str = "[method]shared-context.get";
pub(crate) const SHARED_CONTEXT_ACCESS_LOG_METHOD: &str = "[method]shared-context.log-access";
pub(crate) const SHARED_CONTEXT_TRACE_ID_METHOD: &str = "[method]shared-context.trace-id";
