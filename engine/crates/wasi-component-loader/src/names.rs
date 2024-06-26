pub(crate) static COMPONENT_TYPES: &str = "component:grafbase/types";
pub(crate) static COMPONENT_GATEWAY_REQUEST: &str = "component:grafbase/gateway-request";
pub(crate) static COMPONENT_AUTHORIZATION: &str = "component:grafbase/authorization";

pub(crate) static GATEWAY_HOOK_FUNCTION: &str = "on-gateway-request";
pub(crate) static AUTHORIZATION_HOOK_FUNCTION: &str = "authorized";

pub(crate) static HEADERS_RESOURCE: &str = "headers";
pub(crate) static HEADERS_SET_METHOD: &str = "[method]headers.set";
pub(crate) static HEADERS_GET_METHOD: &str = "[method]headers.get";
pub(crate) static HEADERS_DELETE_METHOD: &str = "[method]headers.delete";

pub(crate) static CONTEXT_RESOURCE: &str = "context";
pub(crate) static CONTEXT_SET_METHOD: &str = "[method]context.set";
pub(crate) static CONTEXT_GET_METHOD: &str = "[method]context.get";
pub(crate) static CONTEXT_DELETE_METHOD: &str = "[method]context.delete";

pub(crate) static SHARED_CONTEXT_RESOURCE: &str = "shared-context";
pub(crate) static SHARED_CONTEXT_GET_METHOD: &str = "[method]shared-context.get";
