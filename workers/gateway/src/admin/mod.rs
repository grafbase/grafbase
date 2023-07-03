use crate::platform::context::RequestContext;
use crate::platform::http::{GqlRequestBuilder, WorkerRequestExt};
use async_graphql::{EmptySubscription, Request as GqlRequest, Schema};
use send_wrapper::SendWrapper;
use serde_json::Value;
use tracing_futures::Instrument;
use worker::{Method, Request, Response, Result, RouteContext};

mod error;
mod graphql;

#[tracing::instrument(err, skip(req, route_context))]
pub async fn handle_graphql_request(mut req: Request, route_context: RouteContext<RequestContext>) -> Result<Response> {
    let request_context = route_context.data;

    if req.method() == Method::Options {
        return Response::empty();
    }

    // deserialize graphql request from incoming http request
    let gql_request = match req.as_gql_request::<GqlRequest>().await {
        Ok(gql_req) => gql_req,
        Err(err) => {
            return Response::error(err.to_string(), http::StatusCode::BAD_REQUEST.as_u16());
        }
    };

    // validate api key auth
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "local"))] {
            use crate::auth::X_API_KEY_HEADER;
            use worker_utils::RequestExt;

            let env = route_context.env;

            let Some(api_key) = req.header_or_query_param(X_API_KEY_HEADER) else {
                return Response::error(format!("Missing {X_API_KEY_HEADER}"), http::StatusCode::BAD_REQUEST.as_u16());
            };

            if let Err(e) = request_context.api_key_auth.verify_api_key(&api_key, &env).await {
                return Response::error(format!("Unauthorized {e}"), http::StatusCode::UNAUTHORIZED.as_u16());
            }
        }
    }

    // use the appropriate cache_provider depending on feature flags
    let global_cache_provider = {
        #[cfg(not(feature = "local"))]
        let provider = crate::cache::CloudflareGlobal::new(request_context.config.cloudflare_config.clone());

        #[cfg(feature = "local")]
        let provider = crate::cache::NoopGlobalCache;

        provider
    };

    // execute admin request
    log::info!(
        request_context.cloudflare_request_context.ray_id,
        "Handling admin request"
    );

    let schema = Schema::build(
        graphql::Query::default(),
        graphql::Mutation::default(),
        EmptySubscription,
    )
    .data(SendWrapper::new(global_cache_provider))
    .data(SendWrapper::new(request_context))
    .finish();

    let response_bytes = {
        let gql_response = schema
            .execute(gql_request)
            .instrument(tracing::info_span!("admin_request"))
            .await;

        serde_json::to_vec(&gql_response)?
    };

    Response::from_bytes(response_bytes)
}

impl GqlRequestBuilder for async_graphql::Request {
    fn new<T: Into<String>>(query: T) -> Self {
        GqlRequest::new(query)
    }

    fn operation_name<T: Into<String>>(&mut self, operation_name: T) {
        self.operation_name = Some(operation_name.into());
    }

    fn variables(&mut self, variables: Value) {
        self.variables = async_graphql::Variables::from_json(variables);
    }
}
