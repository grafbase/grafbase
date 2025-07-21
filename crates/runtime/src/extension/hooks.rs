use std::future::Future;

use error::ErrorResponse;
use http::{request, response};

use crate::extension::ExtensionContext;

pub struct OnRequest<C> {
    pub context: C,
    pub parts: request::Parts,
    pub contract_key: Option<String>,
}

pub trait HooksExtension<Context: ExtensionContext>: Clone + Send + Sync + 'static {
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
