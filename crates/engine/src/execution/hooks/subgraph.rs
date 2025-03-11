use runtime::hooks::{Hooks, SubgraphRequest};

use crate::response::GraphqlError;

impl<H: Hooks> super::RequestHooks<'_, H> {
    pub async fn on_subgraph_request(
        &self,
        subgraph_name: &str,
        request: SubgraphRequest,
    ) -> Result<SubgraphRequest, GraphqlError> {
        self.hooks
            .on_subgraph_request(self.context, subgraph_name, request)
            .await
    }
}
