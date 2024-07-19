use http::HeaderMap;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::SubgraphHooks,
};
use tracing::instrument;
use url::Url;

use super::{guest_error_as_gql, Context, HooksWasi};

impl SubgraphHooks<Context> for HooksWasi {
    #[instrument(skip_all)]
    async fn on_subgraph_request(
        &self,
        context: &Context,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        let Some(ref hooks) = self.0 else {
            return Ok(headers);
        };

        hooks
            .subgraph
            .get()
            .await
            .on_subgraph_request(context.clone(), subgraph_name, method, url, headers)
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::Guest(err) => guest_error_as_gql(err, PartialErrorCode::HookError),
            })
    }
}
