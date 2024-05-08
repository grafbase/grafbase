use std::{collections::BTreeMap, sync::Arc};

use common_types::UdfKind;
use serde::Serialize;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UdfRequestContextRequest {
    pub headers: serde_json::Value,
    pub jwt_claims: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct UdfRequestContext {
    pub request: UdfRequestContextRequest,
}

#[derive(Debug, serde::Serialize)]
pub struct UdfRequest<'a, P: Serialize> {
    pub request_id: &'a str,
    pub name: &'a str,
    pub payload: P,
    pub udf_kind: UdfKind,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum UdfResponse {
    Success(serde_json::Value),
    Error(String),
    GraphQLError {
        message: String,
        #[serde(default)]
        extensions: Option<BTreeMap<String, serde_json::Value>>,
    },
}

#[derive(Clone)]
pub struct UdfInvoker<Payload: Serialize>(Arc<dyn UdfInvokerInner<Payload>>);

impl<P: Serialize> UdfInvoker<P> {
    pub fn new(inner: impl UdfInvokerInner<P> + 'static) -> Self {
        Self(Arc::new(inner))
    }
}

impl<Payload: Serialize> std::ops::Deref for UdfInvoker<Payload> {
    type Target = dyn UdfInvokerInner<Payload>;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[async_trait::async_trait]
pub trait UdfInvokerInner<Payload: Serialize>: Send + Sync {
    async fn invoke(&self, ray_id: &str, request: UdfRequest<'_, Payload>) -> Result<UdfResponse, UdfError>
    where
        Payload: 'async_trait;
}

#[derive(Debug, thiserror::Error)]
pub enum UdfError {
    #[error("Invocation failed")]
    InvocationError,
    #[error("Internal service error")]
    ContractViolation,
}

// Custom resolvers
pub type CustomResolverInvoker = UdfInvoker<CustomResolverRequestPayload>;

#[derive(Debug, serde::Serialize)]
pub struct CustomResolverRequestInfo {}

#[derive(Debug, serde::Serialize)]
pub struct CustomResolverRequestPayload {
    #[serde(rename = "args")]
    pub arguments: std::collections::HashMap<String, serde_json::Value>,
    pub parent: Option<serde_json::Value>,
    pub context: UdfRequestContext,
    pub info: Option<serde_json::Value>,
}

// Authorizer
pub type AuthorizerInvoker = UdfInvoker<AuthorizerRequestPayload>;

// TODO: Switch to serde_tuple and use function.apply(null, args)
#[derive(Debug, serde::Serialize)]
pub struct AuthorizerRequestPayload {
    #[serde(rename = "parent")] // Hack to make it the first argument.
    pub context: UdfRequestContext,
}
