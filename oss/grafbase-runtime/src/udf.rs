use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use grafbase_types::UdfKind;
use serde::Serialize;

#[derive(Debug, serde::Serialize)]
pub struct UdfRequestContextRequest {
    pub headers: serde_json::Value,
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
pub enum CustomResolverResponse {
    Success(serde_json::Value),
    Error(String),
    GraphQLError {
        message: String,
        #[serde(default)]
        extensions: Option<BTreeMap<String, serde_json::Value>>,
    },
}

#[async_trait::async_trait(?Send)]
pub trait UdfInvoker<Payload: Serialize> {
    async fn invoke(
        &self,
        ray_id: &str,
        request: UdfRequest<'_, Payload>,
    ) -> Result<CustomResolverResponse, CustomResolverError>
    where
        Payload: 'async_trait;
}

#[derive(Debug, thiserror::Error)]
pub enum CustomResolverError {
    #[error("Invocation failed")]
    InvocationError,
    #[error("Internal service error")]
    ContractViolation,
}

// Custom resolvers
type BoxedCustomResolversEngineImpl<P> = Box<dyn UdfInvoker<P> + Send + Sync>;

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

#[derive(Clone)]
pub struct CustomResolversEngine {
    inner: Arc<BoxedCustomResolversEngineImpl<CustomResolverRequestPayload>>,
}

impl CustomResolversEngine {
    pub fn new(engine: BoxedCustomResolversEngineImpl<CustomResolverRequestPayload>) -> Self {
        Self {
            inner: Arc::new(engine),
        }
    }
}

impl Deref for CustomResolversEngine {
    type Target = BoxedCustomResolversEngineImpl<CustomResolverRequestPayload>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Authorizer

// TODO: Switch to serde_tuple and use function.apply(null, args)
#[derive(Debug, serde::Serialize)]
pub struct AuthorizerRequestPayload {
    #[serde(rename = "parent")] // Hack to make it the first argument.
    pub context: UdfRequestContext,
}
