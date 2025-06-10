use futures::future::BoxFuture;

use crate::ErrorResponse;

pub(crate) trait HooksInstance {
    fn on_request(&mut self, parts: http::request::Parts)
    -> BoxFuture<'_, Result<http::request::Parts, ErrorResponse>>;

    fn on_response(&mut self, parts: http::response::Parts) -> BoxFuture<'_, anyhow::Result<http::response::Parts>>;
}
