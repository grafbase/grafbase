use runtime::hooks::{ExecutedOperation, ExecutedSubgraphRequest, Hooks};

use crate::response::GraphqlError;

impl<H: Hooks> super::RequestHooks<'_, H> {
    pub async fn on_subgraph_response(
        &self,
        request: ExecutedSubgraphRequest<'_>,
    ) -> Result<H::OnSubgraphResponseOutput, GraphqlError> {
        self.hooks.on_subgraph_response(self.context, request).await
    }

    pub async fn on_operation_response(
        &self,
        operation: ExecutedOperation<'_, H::OnSubgraphResponseOutput>,
    ) -> Result<H::OnOperationResponseOutput, GraphqlError> {
        self.hooks.on_operation_response(self.context, operation).await
    }
}
