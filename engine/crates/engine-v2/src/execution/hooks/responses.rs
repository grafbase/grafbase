use runtime::hooks::{
    ExecutedHttpRequest, ExecutedOperationRequest, ExecutedSubgraphRequest, Hooks, Operation, ResponseHooks,
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
    pub async fn on_operation_response(
        &self,
        operation: Operation<'_>,
        request: ExecutedOperationRequest,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.hooks
            .responses()
            .on_operation_response(self.context, operation, request)
            .await
            .map_err(Into::into)
    }

    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn on_http_response(&self, request: ExecutedHttpRequest) -> Result<(), GraphqlError> {
        self.hooks
            .responses()
            .on_http_response(self.context, request)
            .await
            .map_err(Into::into)
    }
}
