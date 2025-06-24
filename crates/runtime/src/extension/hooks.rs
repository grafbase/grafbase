use std::future::Future;

use error::ErrorResponse;
use http::{request, response};

use super::ExtensionContext;

pub trait HooksExtension: Clone + Send + Sync + 'static {
    type Context: ExtensionContext + Clone + Send + Sync + 'static;

    fn new_context(&self) -> Self::Context;

    fn on_request(
        &self,
        context: &Self::Context,
        parts: request::Parts,
    ) -> impl Future<Output = Result<request::Parts, ErrorResponse>> + Send;

    fn on_response(
        &self,
        context: &Self::Context,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send;
}
