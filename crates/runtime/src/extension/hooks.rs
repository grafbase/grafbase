use std::future::Future;

use error::ErrorResponse;
use http::{request, response};

pub trait HooksExtension<Context>: Clone + Send + Sync + 'static {
    fn on_request(
        &self,
        parts: request::Parts,
    ) -> impl Future<Output = Result<(Context, request::Parts), ErrorResponse>> + Send;

    fn on_response(
        &self,
        context: &Context,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send;
}
