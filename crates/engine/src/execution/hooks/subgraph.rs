use http::HeaderMap;
use runtime::hooks::Hooks;

use crate::response::GraphqlError;

impl<'ctx, H: Hooks> super::RequestHooks<'ctx, H> {
    pub async fn on_subgraph_request(
        &self,
        subgraph_name: &str,
        method: http::Method,
        url: &url::Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, GraphqlError> {
        self.hooks
            .on_subgraph_request(self.context, subgraph_name, method, url, headers)
            .await
            .map_err(Into::into)
    }
}
