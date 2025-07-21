use engine_error::ErrorResponse;
use futures::future::BoxFuture;

use crate::WasmContext;

#[allow(unused_variables)]
pub(crate) trait HooksExtensionInstance {
    fn on_request<'a>(
        &'a mut self,
        context: &'a WasmContext,
        parts: http::request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<http::request::Parts, ErrorResponse>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn on_response(
        &mut self,
        context: WasmContext,
        parts: http::response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<http::response::Parts, String>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
