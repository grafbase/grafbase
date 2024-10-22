use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use grafbase_telemetry::{graphql::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpanBuilder};
use runtime::bytes::OwnedOrSharedBytes;
use schema::{GraphqlEndpointId, GraphqlRootFieldResolverDefinition};
use serde::de::DeserializeSeed;
use tracing::Instrument;
use walker::Walk;

use super::{
    calculate_cache_ttl,
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    request::{execute_subgraph_request, PreparedGraphqlOperation, ResponseIngester, SubgraphVariables},
    SubgraphContext,
};
use crate::{
    execution::{ExecutionError, PlanningResult},
    operation::{OperationType, PlanWalker},
    response::SubgraphResponse,
    sources::{graphql::request::SubgraphGraphqlRequest, ExecutionContext, ExecutionResult, Resolver},
    Runtime,
};

pub(crate) struct GraphqlResolver {
    pub(super) endpoint_id: GraphqlEndpointId,
    pub(super) operation: PreparedGraphqlOperation,
}

impl GraphqlResolver {
    pub fn prepare(
        definition: GraphqlRootFieldResolverDefinition<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Resolver> {
        let operation = PreparedGraphqlOperation::build(operation_type, plan, definition.endpoint_id.into())
            .map_err(|err| format!("Failed to build query: {err}"))?;

        Ok(Resolver::GraphQL(Self {
            endpoint_id: definition.endpoint().id(),
            operation,
        }))
    }

    pub fn build_subgraph_context<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> SubgraphContext<'ctx, R> {
        let endpoint = self.endpoint_id.walk(ctx.schema());
        SubgraphContext::new(
            ctx,
            endpoint,
            SubgraphRequestSpanBuilder {
                subgraph_name: endpoint.subgraph_name(),
                operation_type: self.operation.ty.as_str(),
                sanitized_query: &self.operation.query,
            },
        )
    }

    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let span = ctx.span().entered();
        let variables = SubgraphVariables::<()> {
            plan,
            variables: &self.operation.variables,
            extra_variables: Vec::new(),
        };

        tracing::debug!(
            "Executing request to subgraph named '{}' with query and variables:\n{}\n{}",
            ctx.endpoint().subgraph_name(),
            self.operation.query,
            serde_json::to_string_pretty(&variables).unwrap_or_default()
        );

        let body = serde_json::to_vec(&SubgraphGraphqlRequest {
            query: &self.operation.query,
            variables,
        })
        .map_err(|err| format!("Failed to serialize query: {err}"))?;

        let span = span.exit();
        async {
            let headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            let cache_ttl = ctx.endpoint().config.cache_ttl;
            let cache_key = build_cache_key(ctx.endpoint().subgraph_name(), &body, &headers);

            if let Some((_, cache_key)) = cache_ttl.zip(cache_key.as_ref()) {
                let cache_entry = ctx
                    .engine()
                    .runtime
                    .entity_cache()
                    .get(cache_key)
                    .await
                    .inspect_err(|err| tracing::warn!("Failed to read the cache key {cache_key}: {err}"))
                    .ok()
                    .flatten();

                if let Some(bytes) = cache_entry {
                    ctx.record_cache_hit();

                    let static_ctx = ctx.into_static();
                    let subgraph_response = tokio::task::spawn_blocking(move || {
                        let ctx = &ExecutionContext::from_static(&static_ctx);
                        let response = subgraph_response.as_mut();

                        GraphqlResponseSeed::new(
                            response.next_seed(ctx).ok_or("No object to update")?,
                            RootGraphqlErrors::new(ctx, response),
                        )
                        .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;
                        ExecutionResult::Ok(subgraph_response)
                    })
                    .await
                    .map_err(|err| {
                        tracing::error!("Join error: {err:?}");
                        "Join error"
                    })??;

                    return Ok(subgraph_response);
                } else {
                    ctx.record_cache_miss();
                }
            };

            let ingester = GraphqlIngester {
                ctx: ctx.execution_context(),
                cache_ttl,
                cache_key,
                subgraph_response,
            };

            execute_subgraph_request(ctx, headers, Bytes::from(body), ingester).await
        }
        .instrument(span)
        .await
    }
}

fn build_cache_key(subgraph_name: &str, subgraph_request_body: &[u8], headers: &http::HeaderMap) -> Option<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"v1");
    hasher.update(subgraph_name.as_bytes());
    hasher.update(&headers.len().to_le_bytes());
    for (name, value) in headers {
        hasher.update(&name.as_str().len().to_le_bytes());
        hasher.update(name.as_str().as_bytes());
        hasher.update(&value.len().to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(subgraph_request_body);
    Some(hasher.finalize().to_string())
}

struct GraphqlIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph_response: SubgraphResponse,
    cache_ttl: Option<Duration>,
    cache_key: Option<String>,
}

impl<'ctx, R> ResponseIngester for GraphqlIngester<'ctx, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        let Self {
            ctx,
            mut subgraph_response,
            cache_ttl,
            cache_key,
        } = self;

        let static_ctx = ctx.into_static();
        let (status, subgraph_response, http_response) = tokio::task::spawn_blocking(move || {
            let ctx = ExecutionContext::from_static(&static_ctx);
            let response = subgraph_response.as_mut();
            let status = GraphqlResponseSeed::new(
                response.next_seed(&ctx).ok_or("No object to update")?,
                RootGraphqlErrors::new(&ctx, response),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))?;
            ExecutionResult::Ok((status, subgraph_response, http_response))
        })
        .await
        .map_err(|err| {
            tracing::error!("Join error: {err:?}");
            "Join error"
        })??;

        if let Some(cache_key) = cache_key {
            let cache_ttl = calculate_cache_ttl(status, http_response.headers(), cache_ttl);

            if let Some(cache_ttl) = cache_ttl {
                // We could probably put this call into the background at some point, but for
                // simplicities sake I am not going to do that just now.
                ctx.engine
                    .runtime
                    .entity_cache()
                    .put(&cache_key, Cow::Borrowed(http_response.body().as_ref()), cache_ttl)
                    .await
                    .inspect_err(|err| tracing::warn!("Failed to write the cache key {cache_key}: {err}"))
                    .ok();
            }
        }

        Ok((status, subgraph_response))
    }
}
