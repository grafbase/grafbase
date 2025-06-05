use futures::future::BoxFuture;

use crate::extension::HooksInstance;

impl HooksInstance for super::ExtensionInstanceSince0_15_0 {
    fn on_request(
        &mut self,
        _: http::request::Parts,
    ) -> BoxFuture<'_, Result<http::request::Parts, crate::ErrorResponse>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn on_response(
        &mut self,
        _: http::response::Parts,
    ) -> BoxFuture<'_, Result<http::response::Parts, crate::ErrorResponse>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
