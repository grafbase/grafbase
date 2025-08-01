use engine_error::{ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{OnRequest, ReqwestParts};

use crate::WasmContext;

#[allow(unused_variables)]
pub(crate) trait HooksExtensionInstance {
    fn on_request<'a>(
        &'a mut self,
        context: WasmContext,
        parts: http::request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<OnRequest<WasmContext>, ErrorResponse>>> {
        Box::pin(std::future::ready(Ok(Ok(OnRequest {
            context,
            parts,
            contract_key: None,
        }))))
    }

    fn on_response(
        &mut self,
        context: WasmContext,
        parts: http::response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<http::response::Parts, String>>> {
        Box::pin(std::future::ready(Ok(Ok(parts))))
    }

    fn on_subgraph_request<'a>(
        &'a mut self,
        context: &'a WasmContext,
        parts: ReqwestParts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<ReqwestParts, GraphqlError>>> {
        Box::pin(std::future::ready(Ok(Ok(parts))))
    }
}
