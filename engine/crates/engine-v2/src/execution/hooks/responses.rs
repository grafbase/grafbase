use runtime::hooks::{ExecutedOperation, ExecutedSubgraphRequest, Hooks, ResponseHooks};

use crate::response::GraphqlError;

impl<'ctx, H: Hooks> super::RequestHooks<'ctx, H> {
    pub async fn on_subgraph_response(&self, request: ExecutedSubgraphRequest<'_>) -> Result<Vec<u8>, GraphqlError> {
        self.hooks
            .responses()
            .on_subgraph_response(self.context, request)
            .await
            .map_err(Into::into)
    }

    pub async fn on_operation_response(&self, operation: ExecutedOperation<'_>) -> Result<Vec<u8>, GraphqlError> {
        self.hooks
            .responses()
            .on_operation_response(self.context, operation)
            .await
            .map_err(Into::into)
    }
}
