use http::HeaderMap;
use runtime::hooks::{
    ExecutedGatewayRequest, ExecutedHttpRequest, ExecutedSubgraphRequest, Hooks, Operation, ResponseHooks,
    SubgraphHooks,
};
use tracing::{instrument, Level};

use crate::response::GraphqlError;

impl<'ctx, H: Hooks> super::RequestHooks<'ctx, H> {
    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn on_subgraph_response(&self, request: ExecutedSubgraphRequest<'_>) -> Result<Vec<u8>, GraphqlError> {
        self.hooks
            .responses()
            .on_subgraph_response(self.context, request)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn on_gateway_response(
        &self,
        operation: Operation<'_>,
        request: ExecutedGatewayRequest,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.hooks
            .responses()
            .on_gateway_response(self.context, operation, request)
    }

    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn on_http_response(&self, request: ExecutedHttpRequest<'_>) -> Result<(), GraphqlError> {
        self.hooks.responses().on_http_response(self.context, request)
    }
}
