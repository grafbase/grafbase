use futures::future::BoxFuture;

use crate::{ErrorResponse, SharedContext};

pub(crate) trait HooksInstance {
    fn on_request(
        &mut self,
        context: SharedContext,
        parts: http::request::Parts,
    ) -> BoxFuture<'_, Result<http::request::Parts, ErrorResponse>>;

    fn on_response(
        &mut self,
        context: SharedContext,
        parts: http::response::Parts,
    ) -> BoxFuture<'_, anyhow::Result<http::response::Parts>>;
}
