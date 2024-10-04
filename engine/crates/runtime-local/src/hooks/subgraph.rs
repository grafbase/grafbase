use http::HeaderMap;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    hooks::SubgraphHooks,
};
use tracing::Instrument;
use url::Url;

use super::{guest_error_as_gql, Context, HooksWasi};

impl SubgraphHooks<Context> for HooksWasi {
    async fn on_subgraph_request(
        &self,
        context: &Context,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(headers);
        };

        let Some((mut hook, span)) = inner.get_subgraph_instance("hook: on-subgraph-request").await else {
            return Ok(headers);
        };

        inner
            .run_and_measure(
                "on-subgraph-request",
                hook.on_subgraph_request(inner.shared_context(context), subgraph_name, method, url, headers),
            )
            .instrument(span)
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
