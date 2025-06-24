use std::borrow::Cow;

use bytes::Bytes;
use futures::TryStreamExt;
use futures_util::{StreamExt, stream::BoxStream};
use headers::HeaderMapExt;
use runtime::{
    extension::Data,
    fetch::{FetchRequest, Fetcher},
};
use schema::SubscriptionProtocol;
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
    prepare::{Plan, RootFieldsShapeId},
    response::{GraphqlError, ResponseBuilder, ResponsePartBuilder},
};

impl GraphqlResolver {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + Copy + 'ctx,
    ) -> BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)> {
        let endpoint = ctx.endpoint();
        let shape_id = plan.shape().id;
        let result = match endpoint.subscription_protocol {
            SubscriptionProtocol::ServerSentEvents => self.execute_sse_subscription(ctx, new_response, shape_id).await,
            SubscriptionProtocol::Websocket => {
                let websocket_url = endpoint.websocket_url().unwrap_or_else(|| endpoint.url());

                self.execute_websocket_subscription(ctx, new_response, shape_id, websocket_url)
                    .await
            }
        };
        match result {
            Ok(stream) => stream,
            Err(err) => {
                let mut response = new_response();
                let (_, mut part) = response.create_root_part();
                part.errors.push(err);
                Box::pin(futures_util::stream::once(std::future::ready((response, part))))
            }
        }
    }

    async fn execute_websocket_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
        shape_id: RootFieldsShapeId,
        websocket_url: &'ctx Url,
    ) -> Result<BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>, GraphqlError> {
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

        let request = FetchRequest {
            subgraph_name: endpoint.subgraph_name(),
            url,
            websocket_init_payload: ctx
                .request_context
                .websocket_init_payload
                .as_ref()
                .filter(|_| ctx.schema().settings.websocket_forward_connection_init_payload)
                .cloned(),
            method: http::Method::POST,
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
            let request = request.clone();
            let http_span1 = http_span1.clone();
            async move {
                let result = fetcher
                    .graphql_over_websocket_stream(request)
                    .instrument(http_span1.span())
                    .await;
                (result, None)
            }
        })
        .await;

        let stream = stream.inspect_err(|_| {
            http_span.set_as_http_error(None);
            ctx.set_as_http_error(None);
        })?;

        let stream = stream
            .map_err(move |error| {
                GraphqlError::from(ExecutionError::Fetch {
                    subgraph_name: endpoint.subgraph_name().to_string(),
                    error,
                })
            })
            .map(move |result| {
                let mut response = new_response();
                let (parent_object, part) = response.create_root_part();
                let state = part.into_seed_state(shape_id);

                match result {
                    Ok(data) => {
                        let seed = GraphqlResponseSeed::new(
                            state.parent_seed(&parent_object),
                            GraphqlErrorsSeed::new(&state, convert_root_error_path),
                        );
                        if let Err(Some(error)) = state.deserialize_data_with(data, seed) {
                            state.insert_error_update(&parent_object, [error]);
                        }
                    }
                    Err(error) => state.insert_error_update(&parent_object, [error]),
                }

                (response, state.into_response_part())
            });

        Ok(Box::pin(stream))
    }

    async fn execute_sse_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
        shape_id: RootFieldsShapeId,
    ) -> Result<BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>, GraphqlError> {
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
            .map_err(|err| {
                tracing::error!("Failed to serialize query: {err}");
                GraphqlError::internal_server_error()
            })?;

            let mut headers = ctx.subgraph_headers_with_rules(endpoint.header_rules());

            headers.typed_insert(headers::ContentType::json());
            headers.typed_insert(headers::ContentLength(body.len() as u64));

            FetchRequest {
                websocket_init_payload: None,
                subgraph_name: endpoint.subgraph_name(),
                url: Cow::Owned(endpoint.url().clone()),
                method: http::Method::POST,
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
            let request = request.clone();
            let http_span1 = http_span1.clone();
            async move {
                let result = fetcher
                    .graphql_over_sse_stream(request)
                    .instrument(http_span1.span())
                    .await;
                (result, None)
            }
        })
        .await;

        let stream = stream.inspect_err(|err| {
            http_span.set_as_http_error(err.as_fetch_invalid_status_code());
            ctx.set_as_http_error(err.as_fetch_invalid_status_code());
        })?;

        let stream = stream
            .map_err(move |error| {
                GraphqlError::from(ExecutionError::Fetch {
                    subgraph_name: endpoint.subgraph_name().to_string(),
                    error,
                })
            })
            .map(move |result| {
                let mut response = new_response();
                let (parent_object, part) = response.create_root_part();
                let state = part.into_seed_state(shape_id);

                match result {
                    Ok(bytes) => {
                        let seed = GraphqlResponseSeed::new(
                            state.parent_seed(&parent_object),
                            GraphqlErrorsSeed::new(&state, convert_root_error_path),
                        );
                        if let Err(Some(error)) = state.deserialize_data_with(&Data::Json(bytes), seed) {
                            state.insert_error_update(&parent_object, [error]);
                        }
                    }
                    Err(error) => state.insert_error_update(&parent_object, [error]),
                }

                (response, state.into_response_part())
            });

        Ok(Box::pin(stream))
    }
}
