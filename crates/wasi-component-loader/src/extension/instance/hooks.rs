use std::sync::Arc;

use engine::EngineOperationContext;
use engine_error::{ErrorResponse, GraphqlError};
use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use event_queue::EventQueue;
use futures::future::BoxFuture;
use runtime::extension::{ExtensionRequestContext, OnRequest, ReqwestParts};

#[allow(unused_variables)]
pub(crate) trait HooksExtensionInstance {
    fn on_request<'a>(
        &'a mut self,
        event_queue: EventQueue,
        parts: http::request::Parts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<OnRequest, ErrorResponse>>> {
        Box::pin(std::future::ready(Ok(Ok(OnRequest {
            parts,
            contract_key: None,
            context: ExtensionRequestContext {
                event_queue: Arc::new(event_queue),
                hooks_context: Default::default(),
            },
        }))))
    }

    fn on_response(
        &mut self,
        ctx: ExtensionRequestContext,
        parts: http::response::Parts,
    ) -> BoxFuture<'_, wasmtime::Result<Result<http::response::Parts, String>>> {
        Box::pin(std::future::ready(Ok(Ok(parts))))
    }

    fn on_graphql_subgraph_request<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        subgraph: GraphqlSubgraph<'a>,
        parts: ReqwestParts,
    ) -> BoxFuture<'a, wasmtime::Result<Result<ReqwestParts, GraphqlError>>> {
        Box::pin(std::future::ready(Ok(Ok(parts))))
    }

    fn on_virtual_subgraph_request<'a>(
        &'a mut self,
        ctx: EngineOperationContext,
        subgraph: VirtualSubgraph<'a>,
        headers: http::HeaderMap,
    ) -> BoxFuture<'a, wasmtime::Result<Result<http::HeaderMap, GraphqlError>>> {
        Box::pin(std::future::ready(Ok(Ok(headers))))
    }
}
