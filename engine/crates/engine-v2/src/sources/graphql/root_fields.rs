use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use grafbase_telemetry::{gql_response_status::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpan};
use runtime::{bytes::OwnedOrSharedBytes, hooks::CacheStatus};
use schema::sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker, RootFieldResolverDefinitionWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use super::{
    calculate_cache_ttl,
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    request::{execute_subgraph_request, PreparedGraphqlOperation, ResponseIngester, SubgraphVariables},
};
use crate::{
    execution::PlanningResult,
    operation::{OperationType, PlanWalker},
    response::SubgraphResponse,
    sources::{
        graphql::{record, request::SubgraphGraphqlRequest},
        ExecutionContext, ExecutionResult, Resolver, SubgraphRequestContext,
    },
    Runtime,
};

pub(crate) struct GraphqlResolver {
    pub(super) endpoint_id: GraphqlEndpointId,
    pub(super) operation: PreparedGraphqlOperation,
}

impl GraphqlResolver {
    pub fn prepare(
        definition: RootFieldResolverDefinitionWalker<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Resolver> {
        let operation = PreparedGraphqlOperation::build(operation_type, plan)
            .map_err(|err| format!("Failed to build query: {err}"))?;

        Ok(Resolver::GraphQL(Self {
            endpoint_id: definition.endpoint().id(),
            operation,
        }))
    }

    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphRequestContext<'ctx, R>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let variables = SubgraphVariables::<()> {
            plan: ctx.plan(),
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

        let headers = ctx
            .execution_context()
            .subgraph_headers_with_rules(ctx.endpoint().header_rules());

        let subgraph_cache_ttl = ctx.endpoint().entity_cache_ttl();
        let cache_key = build_cache_key(ctx.endpoint().subgraph_name(), &body, &headers);

        if let Some((_, cache_key)) = subgraph_cache_ttl.zip(cache_key.as_ref()) {
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
                ctx.request_info().set_cache_status(CacheStatus::Hit);
                record::record_subgraph_cache_hit(ctx.execution_context(), ctx.endpoint());

                let response = subgraph_response.as_mut();

                GraphqlResponseSeed::new(
                    response
                        .next_seed(ctx.execution_context())
                        .ok_or("No object to update")?,
                    RootGraphqlErrors::new(ctx.execution_context(), response),
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

                return Ok(subgraph_response);
            } else {
                ctx.request_info().set_cache_status(CacheStatus::Miss);
                record::record_subgraph_cache_miss(ctx.execution_context(), ctx.endpoint());
            }
        };

        let span = SubgraphRequestSpan {
            name: ctx.endpoint().subgraph_name(),
            operation_type: self.operation.ty.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: ctx.endpoint().url(),
        }
        .into_span();

        let ingester = GraphqlIngester {
            ctx: ctx.execution_context(),
            subgraph_cache_ttl,
            cache_key,
            subgraph_response,
        };

        execute_subgraph_request(ctx, span.clone(), headers, Bytes::from(body), ingester)
            .instrument(span)
            .await
    }

    pub fn endpoint<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> GraphqlEndpointWalker<'ctx> {
        ctx.schema().walk(self.endpoint_id)
    }

    pub fn operation_type(&self) -> OperationType {
        self.operation.ty
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
    subgraph_cache_ttl: Option<Duration>,
    cache_key: Option<String>,
}

impl<'ctx, R> ResponseIngester for GraphqlIngester<'ctx, R>
where
    R: Runtime,
{
    async fn ingest(
        mut self,
        http_response: http::Response<OwnedOrSharedBytes>,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), crate::execution::ExecutionError> {
        let status = {
            let response = self.subgraph_response.as_mut();
            GraphqlResponseSeed::new(
                response.next_seed(self.ctx).ok_or("No object to update")?,
                RootGraphqlErrors::new(self.ctx, response),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))?
        };

        if let Some(cache_key) = self.cache_key {
            let cache_ttl = calculate_cache_ttl(status, http_response.headers(), self.subgraph_cache_ttl);

            if let Some(cache_ttl) = cache_ttl {
                // We could probably put this call into the background at some point, but for
                // simplicities sake I am not going to do that just now.
                self.ctx
                    .engine
                    .runtime
                    .entity_cache()
                    .put(&cache_key, Cow::Borrowed(http_response.body().as_ref()), cache_ttl)
                    .await
                    .inspect_err(|err| tracing::warn!("Failed to write the cache key {cache_key}: {err}"))
                    .ok();
            }
        }

        Ok((status, self.subgraph_response))
    }
}
