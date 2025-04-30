use std::borrow::Cow;

use bytes::Bytes;
use futures::{FutureExt, TryStreamExt};
use futures_util::{StreamExt, stream::BoxStream};
use headers::HeaderMapExt;
use runtime::fetch::{FetchRequest, Fetcher};
use schema::SubscriptionProtocol;
use serde::de::DeserializeSeed;
use tracing::Instrument;
use url::Url;

use super::{
    GraphqlResolver, SubgraphContext, convert_root_error_path,
    deserialize::{GraphqlErrorsSeed, GraphqlResponseSeed},
    request::{SubgraphGraphqlRequest, SubgraphVariables, retrying_fetch},
};
use crate::{
    Runtime,
    execution::ExecutionError,
    prepare::{ConcreteShapeId, Plan},
    resolver::ExecutionResult,
    response::{GraphqlError, ResponseBuilder, ResponsePartBuilder},
};

impl GraphqlResolver {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<(ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>>> {
        let endpoint = ctx.endpoint();
        let shape_id = plan.shape_id();
        match endpoint.subscription_protocol {
            SubscriptionProtocol::ServerSentEvents => self.execute_sse_subscription(ctx, shape_id, new_response).await,
            SubscriptionProtocol::Websocket => {
                let websocket_url = endpoint.websocket_url().unwrap_or_else(|| endpoint.url());

                self.execute_websocket_subscription(ctx, shape_id, new_response, websocket_url)
                    .await
            }
        }
    }

    async fn execute_websocket_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        shape_id: ConcreteShapeId,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
        websocket_url: &'ctx Url,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<(ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>>> {
        let endpoint = ctx.endpoint();

        // If the user doesn't provide an explicit websocket URL we use the normal URL,
        // so make sure to convert the scheme to something appropriate
        let url = match websocket_url.scheme() {
            "http" => {
                let mut url = websocket_url.clone();
                url.set_scheme("ws").expect("this to work");
                Cow::Owned(url)
            }
            "https" => {
                let mut url = websocket_url.clone();
                url.set_scheme("wss").expect("this to work");
                Cow::Owned(url)
            }
            _ => Cow::Borrowed(websocket_url),
        };

        let headers = ctx.subgraph_headers_with_rules(endpoint.header_rules());
        let req = runtime::hooks::SubgraphRequest {
            method: http::Method::POST,
            url: url.into_owned(),
            headers,
        };
        let runtime::hooks::SubgraphRequest { method, url, headers } =
            ctx.hooks().on_subgraph_request(endpoint.subgraph_name(), req).await?;

        let request = FetchRequest {
            subgraph_name: endpoint.subgraph_name(),
            url: Cow::Owned(url),
            websocket_init_payload: ctx
                .request_context
                .websocket_init_payload
                .as_ref()
                .filter(|_| ctx.schema().settings.websocket_forward_connection_init_payload)
                .cloned(),
            method,
            headers,
            body: &SubgraphGraphqlRequest {
                query: &self.subgraph_operation.query,
                variables: SubgraphVariables::<()> {
                    ctx: ctx.input_value_context(),
                    variables: &self.subgraph_operation.variables,
                    extra_variables: Vec::new(),
                },
            },
            timeout: endpoint.config.timeout,
        };

        let fetcher = ctx.runtime().fetcher();
        let http_span = ctx.create_subgraph_request_span(&request);
        let http_span1 = http_span.clone();

        let stream = retrying_fetch(ctx, move || {
            fetcher
                .graphql_over_websocket_stream(request.clone())
                .then(|res| async { (res, None) })
                .instrument(http_span1.span())
        })
        .await;

        let stream = stream.inspect_err(|_| {
            http_span.set_as_http_error(None);
            ctx.set_as_http_error(None);
        })?;

        let stream = stream
            .map_err(move |error| ExecutionError::Fetch {
                subgraph_name: endpoint.subgraph_name().to_string(),
                error,
            })
            .map(move |subgraph_response| {
                let mut subscription_response = new_response();
                let (input_id, part) = subscription_response.create_root_part();
                let part = part.into_shared();

                GraphqlResponseSeed::new(
                    part.seed(shape_id, input_id),
                    GraphqlErrorsSeed::new(part.clone(), convert_root_error_path),
                )
                .deserialize(subgraph_response?)
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subscription response: {}", err);
                    GraphqlError::invalid_subgraph_response()
                })?;

                Ok((subscription_response, part.unshare().unwrap()))
            });

        Ok(Box::pin(stream))
    }

    async fn execute_sse_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        shape_id: ConcreteShapeId,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<(ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>>> {
        let endpoint = ctx.endpoint();

        let request = {
            let body = sonic_rs::to_vec(&SubgraphGraphqlRequest {
                query: &self.subgraph_operation.query,
                variables: SubgraphVariables::<()> {
                    ctx: ctx.input_value_context(),
                    variables: &self.subgraph_operation.variables,
                    extra_variables: Vec::new(),
                },
            })
            .map_err(|err| format!("Failed to serialize query: {err}"))?;

            let headers = ctx.subgraph_headers_with_rules(endpoint.header_rules());
            let req = runtime::hooks::SubgraphRequest {
                method: http::Method::POST,
                url: endpoint.url().clone(),
                headers,
            };
            let runtime::hooks::SubgraphRequest {
                method,
                url,
                mut headers,
            } = ctx.hooks().on_subgraph_request(endpoint.subgraph_name(), req).await?;

            headers.typed_insert(headers::ContentType::json());
            headers.typed_insert(headers::ContentLength(body.len() as u64));
            FetchRequest {
                websocket_init_payload: None,
                subgraph_name: endpoint.subgraph_name(),
                url: Cow::Owned(url),
                method,
                headers,
                body: Bytes::from(body),
                timeout: endpoint.config.timeout,
            }
        };

        ctx.record_request_size(&request);

        let http_span = ctx.create_subgraph_request_span(&request);
        let fetcher = ctx.runtime().fetcher();

        let http_span1 = http_span.clone();
        let stream = retrying_fetch(ctx, move || {
            fetcher
                .graphql_over_sse_stream(request.clone())
                .then(|result| async { (result, None) })
                .instrument(http_span1.span())
        })
        .await;

        let stream = stream.inspect_err(|err| {
            http_span.set_as_http_error(err.as_fetch_invalid_status_code());
            ctx.set_as_http_error(err.as_fetch_invalid_status_code());
        })?;

        let stream = stream
            .map_err(move |error| ExecutionError::Fetch {
                subgraph_name: endpoint.subgraph_name().to_string(),
                error,
            })
            .map(move |subgraph_response| {
                let mut response = new_response();
                let (root_object_id, part) = response.create_root_part();
                let part = part.into_shared();

                GraphqlResponseSeed::new(
                    part.seed(shape_id, root_object_id),
                    GraphqlErrorsSeed::new(part.clone(), convert_root_error_path),
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(&subgraph_response?))
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subscription response: {}", err);
                    GraphqlError::invalid_subgraph_response()
                })?;

                Ok((response, part.unshare().unwrap()))
            });

        Ok(Box::pin(stream))
    }
}
