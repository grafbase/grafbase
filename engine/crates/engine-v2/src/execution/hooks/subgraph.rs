use http::HeaderMap;
use runtime::hooks::{Hooks, SubgraphHooks};
use tracing::{instrument, Level};

use crate::response::GraphqlError;

impl<'ctx, H: Hooks> super::RequestHooks<'ctx, H> {
    /// A hook called just before executing a subgraph request.
    ///
    /// # Arguments
    ///
    /// * `subgraph_name` - The name of the subgraph being requested.
    /// * `method` - The HTTP method of the request (e.g., GET, POST).
    /// * `url` - The URL for the subgraph request.
    /// * `headers` - The headers associated with the subgraph request.
    ///
    /// # Returns
    ///
    /// Returns a result containing the headers if the subgraph request should continue, or an
    /// error if the execution should abort.
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
