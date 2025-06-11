use futures::future::BoxFuture;

use crate::{SharedContext, extension::HooksInstance};

impl HooksInstance for super::ExtensionInstanceSince090 {
    fn on_request(
        &mut self,
        _: SharedContext,
        _: http::request::Parts,
    ) -> BoxFuture<'_, Result<http::request::Parts, crate::ErrorResponse>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn on_response(
        &mut self,
        _: SharedContext,
        _: http::response::Parts,
    ) -> BoxFuture<'_, anyhow::Result<http::response::Parts>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
