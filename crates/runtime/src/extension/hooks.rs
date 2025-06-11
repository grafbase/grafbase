use std::future::Future;

use error::ErrorResponse;
use http::{request, response};

pub trait HooksExtension: Send + Sync + 'static {
    type Context: Clone + Send + Sync + 'static;

    fn new_context(&self) -> Self::Context;

    fn on_request(
        &self,
        context: Self::Context,
        parts: request::Parts,
    ) -> impl Future<Output = Result<request::Parts, ErrorResponse>> + Send;

    fn on_response(
        &self,
        context: Self::Context,
        parts: response::Parts,
    ) -> impl Future<Output = Result<response::Parts, String>> + Send;
}

impl HooksExtension for () {
    type Context = ();

    fn new_context(&self) -> Self::Context {}

    async fn on_request(&self, _: Self::Context, parts: request::Parts) -> Result<request::Parts, ErrorResponse> {
        Ok(parts)
    }

    async fn on_response(&self, _: Self::Context, parts: response::Parts) -> Result<response::Parts, String> {
        Ok(parts)
    }
}
