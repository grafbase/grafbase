use std::{borrow::Cow, sync::Arc, time::Duration};

use grafbase_telemetry::{graphql::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpanBuilder};
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
    prepare::{ConcreteShapeId, Plan, PlanError, PlanResult, PrepareContext, SubgraphSelectionSet},
    resolver::{ExecutionResult, graphql::request::SubgraphGraphqlRequest},
    response::{ErrorPath, ErrorPathSegment, GraphqlError, ParentObjectId, ParentObjects, ResponsePartBuilder},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct GraphqlResolver {
    pub endpoint_id: GraphqlEndpointId,
    pub subgraph_operation: PreparedGraphqlOperation,
}

impl GraphqlResolver {
    pub fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: GraphqlRootFieldResolverDefinition<'_>,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> PlanResult<Self> {
        let parent_object = selection_set
            .fields()
            .next()
            .and_then(|field| field.definition().parent_entity().as_object())
            // FIXME: this is a workaround, we likely require a __typename which should even reach
            // this resolver.
            .unwrap_or_else(|| ctx.schema().query());

        let subgraph_operation =
            PreparedGraphqlOperation::build(ctx.schema(), definition.endpoint_id, parent_object, selection_set)
                .map_err(|err| {
                    tracing::error!("Failed to build query: {err}");
                    PlanError::Internal
                })?;
        Ok(Self {
            endpoint_id: definition.endpoint().id,
            subgraph_operation,
        })
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
        plan: Plan<'ctx>,
        parent_objects: Arc<ParentObjects>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> ExecutionResult<ResponsePartBuilder<'ctx>> {
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
            sonic_rs::to_string_pretty(&variables).unwrap_or_default()
        );

        let body = sonic_rs::to_vec(&SubgraphGraphqlRequest {
            query: &self.subgraph_operation.query,
            variables,
        })
        .map_err(|err| format!("Failed to serialize query: {err}"))?;

        let span = span.exit();
        let shape_id = plan.shape_id();
        let parent_object_id = parent_objects.ids().next().ok_or("No object to update")?;
        async {
            let subgraph_headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            if ctx.endpoint().config.cache_ttl.is_some() {
                fetch_response_with_cache(ctx, subgraph_headers, body, parent_object_id, shape_id, response_part).await
            } else {
                fetch_response_without_cache(ctx, subgraph_headers, body, parent_object_id, shape_id, response_part)
                    .await
            }
        }
        .instrument(span)
        .await
    }
}

async fn fetch_response_without_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    parent_object_id: ParentObjectId,
    shape_id: ConcreteShapeId,
    response_part: ResponsePartBuilder<'ctx>,
) -> ExecutionResult<ResponsePartBuilder<'ctx>> {
    struct Ingester {
        parent_object_id: ParentObjectId,
        shape_id: ConcreteShapeId,
    }

    impl ResponseIngester for Ingester {
        async fn ingest(
            self,
            http_response: http::Response<OwnedOrSharedBytes>,
            response_part: ResponsePartBuilder<'_>,
        ) -> Result<(GraphqlResponseStatus, ResponsePartBuilder<'_>), ExecutionError> {
            let response_part = response_part.into_shared();
            let status = {
                GraphqlResponseSeed::new(
                    response_part.seed(self.shape_id, self.parent_object_id),
                    GraphqlErrorsSeed::new(response_part.clone(), convert_root_error_path),
                )
                .deserialize(&mut sonic_rs::Deserializer::from_slice(http_response.body()))
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subgraph response: {}", err);
                    GraphqlError::invalid_subgraph_response()
                })?
            };

            Ok((status, response_part.unshare().unwrap()))
        }
    }

    execute_subgraph_request(
        ctx,
        subgraph_headers,
        body,
        response_part,
        Ingester {
            parent_object_id,
            shape_id,
        },
    )
    .await
}

async fn fetch_response_with_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    parent_object_id: ParentObjectId,
    shape_id: ConcreteShapeId,
    response_part: ResponsePartBuilder<'ctx>,
) -> ExecutionResult<ResponsePartBuilder<'ctx>> {
    match super::cache::fetch_response(ctx, &subgraph_headers, &body).await {
        Ok(ResponseCacheHit { data }) => {
            ctx.record_cache_hit();

            let response_part = response_part.into_shared();
            GraphqlResponseSeed::new(
                response_part.seed(shape_id, parent_object_id),
                GraphqlErrorsSeed::new(response_part.clone(), convert_root_error_path),
            )
            .deserialize(&mut sonic_rs::Deserializer::from_slice(&data))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?;

            Ok(response_part.unshare().unwrap())
        }
        Err(ResponseCacheMiss { key }) => {
            ctx.record_cache_miss();
            let ingester = GraphqlWithCachePutIngester {
                ctx: ctx.execution_context(),
                parent_object_id,
                subgraph_default_cache_ttl: ctx.endpoint().config.cache_ttl,
                cache_key: key,
                shape_id,
            };

            execute_subgraph_request(ctx, subgraph_headers, body, response_part, ingester).await
        }
    }
}

struct GraphqlWithCachePutIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    parent_object_id: ParentObjectId,
    shape_id: ConcreteShapeId,
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
        response_part: ResponsePartBuilder<'_>,
    ) -> Result<(GraphqlResponseStatus, ResponsePartBuilder<'_>), ExecutionError> {
        let Self {
            ctx,
            shape_id,
            parent_object_id,
            subgraph_default_cache_ttl,
            cache_key,
        } = self;

        let (status, response_part) = {
            let response_part = response_part.into_shared();
            let status = GraphqlResponseSeed::new(
                response_part.seed(shape_id, parent_object_id),
                GraphqlErrorsSeed::new(response_part.clone(), convert_root_error_path),
            )
            .deserialize(&mut sonic_rs::Deserializer::from_slice(http_response.body()))
            .map_err(|err| {
                tracing::error!("Failed to deserialize subscription response: {}", err);
                GraphqlError::invalid_subgraph_response()
            })?;
            (status, response_part.unshare().unwrap())
        };

        if status.is_success() {
            let cache_ttl =
                super::cache::calculate_cache_ttl(status, http_response.headers(), subgraph_default_cache_ttl);
            if let Some(cache_ttl) = cache_ttl {
                // We could probably put this call into the background at some point, but for
                // simplicities sake I am not going to do that just now.
                ctx.runtime()
                    .entity_cache()
                    .put(&cache_key, Cow::Borrowed(http_response.body().as_ref()), cache_ttl)
                    .await
                    .inspect_err(|err| tracing::warn!("Failed to write the cache key {cache_key}: {err}"))
                    .ok();
            }
        }

        Ok((status, response_part))
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
                out.push(ErrorPathSegment::UnknownField(field.into_boxed_str()));
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
