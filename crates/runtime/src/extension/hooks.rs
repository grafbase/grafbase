use std::future::Future;

use error::ErrorResponse;
use http::{request, response};

pub trait HooksExtension: Send + Sync + 'static {
    type Context: Send + Sync + 'static;

    fn on_request(&self, parts: request::Parts) -> impl Future<Output = Result<request::Parts, ErrorResponse>> + Send;

    fn on_response(&self, parts: response::Parts) -> impl Future<Output = Result<response::Parts, String>> + Send;
}

impl HooksExtension for () {
    type Context = ();

    async fn on_request(&self, parts: request::Parts) -> Result<request::Parts, ErrorResponse> {
        Ok(parts)
    }

    async fn on_response(&self, parts: response::Parts) -> Result<response::Parts, String> {
        Ok(parts)
    }
}
