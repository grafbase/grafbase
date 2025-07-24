use std::future::Future;

use error::{ErrorResponse, GraphqlError};
use http::{request, response};
use url::Url;

use crate::extension::ExtensionContext;

pub struct OnRequest<C> {
    pub context: C,
    pub parts: request::Parts,
    pub contract_key: Option<String>,
}

pub trait GatewayHooksExtension<Context: ExtensionContext>: Clone + Send + Sync + 'static {
    fn on_request(
        &self,
        parts: request::Parts,
    ) -> impl Future<Output = Result<OnRequest<Context>, ErrorResponse>> + Send;

    fn on_response(
        &self,
        context: Context,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send;
}

pub struct ReqwestParts {
    pub url: Url,
    pub method: http::Method,
    pub headers: http::HeaderMap,
}

pub trait EngineHooksExtension<Context: ExtensionContext>: Send + Sync + 'static {
    fn on_subgraph_request(
        &self,
        context: &Context,
        parts: ReqwestParts,
    ) -> impl Future<Output = Result<ReqwestParts, GraphqlError>> + Send;
}
