use std::{borrow::Cow, sync::Arc, time::Duration};

use grafbase_telemetry::{
    graphql::{GraphqlResponseStatus, OperationType},
    span::subgraph::SubgraphRequestSpanBuilder,
};
use runtime::bytes::OwnedOrSharedBytes;
use schema::{GraphqlEndpointId, GraphqlRootFieldResolverDefinition};
use serde::de::DeserializeSeed;
use tracing::Instrument;
use walker::Walk;

use super::{
    SubgraphContext,
    cache::{ResponseCacheHit, ResponseCacheMiss},
    deserialize::{GraphqlErrorsSeed, GraphqlResponseSeed},
    request::{PreparedGraphqlOperation, ResponseIngester, SubgraphVariables, execute_subgraph_request},
};
use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionError},
    prepare::{PlanError, PlanQueryPartition, PlanResult},
    resolver::{ExecutionResult, Resolver, graphql::request::SubgraphGraphqlRequest},
    response::{ErrorPath, ErrorPathSegment, GraphqlError, InputObjectId, InputResponseObjectSet, SubgraphResponse},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct GraphqlResolver {
    pub endpoint_id: GraphqlEndpointId,
    pub subgraph_operation: PreparedGraphqlOperation,
}

impl GraphqlResolver {
    pub fn prepare(
        definition: GraphqlRootFieldResolverDefinition<'_>,
        operation_type: OperationType,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Resolver> {
        let subgraph_operation =
            PreparedGraphqlOperation::build(operation_type, plan_query_partition).map_err(|err| {
                tracing::error!("Failed to build query: {err}");
                PlanError::InternalError
            })?;

        Ok(Resolver::Graphql(Self {
            endpoint_id: definition.endpoint().id,
            subgraph_operation,
        }))
    }

    pub fn build_subgraph_context<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> SubgraphContext<'ctx, R> {
        let endpoint = self.endpoint_id.walk(ctx.schema());
        SubgraphContext::new(
            ctx,
            endpoint,
            SubgraphRequestSpanBuilder {
                subgraph_name: endpoint.subgraph_name(),
                operation_type: self.subgraph_operation.ty.as_str(),
                sanitized_query: &self.subgraph_operation.query,
            },
        )
    }

    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphContext<'ctx, R>,
        input_object_refs: Arc<InputResponseObjectSet>,
        subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let span = ctx.span().entered();
        let variables = SubgraphVariables::<()> {
            ctx: ctx.input_value_context(),
            variables: &self.subgraph_operation.variables,
            extra_variables: Vec::new(),
        };

        tracing::debug!(
            "Executing request to subgraph named '{}' with query and variables:\n{}\n{}",
            ctx.endpoint().subgraph_name(),
            self.subgraph_operation.query,
            serde_json::to_string_pretty(&variables).unwrap_or_default()
        );

        let body = serde_json::to_vec(&SubgraphGraphqlRequest {
            query: &self.subgraph_operation.query,
            variables,
        })
        .map_err(|err| format!("Failed to serialize query: {err}"))?;

        let span = span.exit();
        let input_object_id = input_object_refs.ids().next().ok_or("No object to update")?;
        async {
            let subgraph_headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            if ctx.endpoint().config.cache_ttl.is_some() {
                fetch_response_with_cache(ctx, subgraph_headers, body, input_object_id, subgraph_response).await
            } else {
                fetch_response_without_cache(ctx, subgraph_headers, body, input_object_id, subgraph_response).await
            }
        }
        .instrument(span)
        .await
    }
}

async fn fetch_response_without_cache<R: Runtime>(
    ctx: &mut SubgraphContext<'_, R>,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    input_object_id: InputObjectId,
    mut subgraph_response: SubgraphResponse,
) -> ExecutionResult<SubgraphResponse> {
    let execution_ctx = ctx.execution_context();
    execute_subgraph_request(
        ctx,
        subgraph_headers,
        body,
        move |http_response: http::Response<OwnedOrSharedBytes>| {
            let status = {
                let response = subgraph_response.as_shared_mut();
                GraphqlResponseSeed::new(
                    response.seed(&execution_ctx, input_object_id),
                    GraphqlErrorsSeed::new(response, convert_root_error_path),
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subgraph response: {}", err);
                    GraphqlError::invalid_subgraph_response()
                })?
            };

            Ok((status, subgraph_response))
        },
    )
    .await
}

async fn fetch_response_with_cache<R: Runtime>(
    ctx: &mut SubgraphContext<'_, R>,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    input_object_id: InputObjectId,
    mut subgraph_response: SubgraphResponse,
) -> ExecutionResult<SubgraphResponse> {
    match super::cache::fetch_response(ctx, &subgraph_headers, &body).await {
        Ok(ResponseCacheHit { data }) => {
            ctx.record_cache_hit();

            let response = subgraph_response.as_shared_mut();
            GraphqlResponseSeed::new(
                response.seed(ctx, input_object_id),
                GraphqlErrorsSeed::new(response, convert_root_error_path),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&data))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?;

            Ok(subgraph_response)
        }
        Err(ResponseCacheMiss { key }) => {
            ctx.record_cache_miss();
            let ingester = GraphqlWithCachePutIngester {
                ctx: ctx.execution_context(),
                input_object_id,
                subgraph_default_cache_ttl: ctx.endpoint().config.cache_ttl,
                cache_key: key,
                subgraph_response,
            };

            execute_subgraph_request(ctx, subgraph_headers, body, ingester).await
        }
    }
}

struct GraphqlWithCachePutIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    input_object_id: InputObjectId,
    subgraph_response: SubgraphResponse,
    subgraph_default_cache_ttl: Option<Duration>,
    cache_key: String,
}

impl<R> ResponseIngester for GraphqlWithCachePutIngester<'_, R>
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
            input_object_id,
            subgraph_default_cache_ttl,
            cache_key,
        } = self;

        let status = {
            let response = subgraph_response.as_shared_mut();
            GraphqlResponseSeed::new(
                response.seed(&ctx, input_object_id),
                GraphqlErrorsSeed::new(response, convert_root_error_path),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subscription response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?
        };

        if status.is_success() {
            let cache_ttl =
                super::cache::calculate_cache_ttl(status, http_response.headers(), subgraph_default_cache_ttl);
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

pub(super) fn convert_root_error_path(path: serde_json::Value) -> Option<ErrorPath> {
    let mut out = Vec::new();
    let serde_json::Value::Array(path) = path else {
        return None;
    };
    for segment in path {
        match segment {
            serde_json::Value::String(field) => {
                out.push(ErrorPathSegment::UnknownField(field));
            }
            serde_json::Value::Number(index) => {
                out.push(ErrorPathSegment::Index(index.as_u64()? as usize));
            }
            _ => {
                return None;
            }
        }
    }
    Some(out.into())
}
