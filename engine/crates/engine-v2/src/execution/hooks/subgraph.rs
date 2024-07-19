use http::HeaderMap;
use runtime::hooks::{Hooks, SubgraphHooks};
use tracing::{instrument, Level};

use crate::response::GraphqlError;

impl<'ctx, H: Hooks> super::RequestHooks<'ctx, H> {
    #[instrument(skip_all, ret(level = Level::DEBUG))]
    pub async fn on_subgraph_request(
        &self,
        subgraph_name: &str,
        method: http::Method,
        url: &url::Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, GraphqlError> {
        self.hooks
            .subgraph()
            .on_subgraph_request(self.context, subgraph_name, method, url, headers)
            .await
            .map_err(Into::into)
    }
}
