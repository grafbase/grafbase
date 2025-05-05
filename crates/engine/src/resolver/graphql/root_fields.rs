use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use grafbase_telemetry::{graphql::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpanBuilder};
use schema::{GraphqlEndpointId, GraphqlRootFieldResolverDefinition};
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
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanResult, PrepareContext, RootFieldsShapeId, SubgraphSelectionSet},
    resolver::graphql::request::SubgraphGraphqlRequest,
    response::{Deserializable, ErrorPath, ErrorPathSegment, GraphqlError, ParentObjects, ResponsePartBuilder},
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
        parent_objects: ParentObjects,
        mut response_part: ResponsePartBuilder<'ctx>,
    ) -> ResponsePartBuilder<'ctx> {
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

        let body = match sonic_rs::to_vec(&SubgraphGraphqlRequest {
            query: &self.subgraph_operation.query,
            variables,
        }) {
            Ok(body) => body,
            Err(err) => {
                tracing::error!("Failed to serialize query: {err}");
                response_part.insert_error_updates(
                    &parent_objects,
                    plan.shape().id,
                    GraphqlError::internal_server_error(),
                );
                return response_part;
            }
        };

        let span = span.exit();
        async {
            let subgraph_headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            if ctx.endpoint().config.cache_ttl.is_some() {
                fetch_response_with_cache(
                    ctx,
                    parent_objects,
                    subgraph_headers,
                    body,
                    plan.shape().id,
                    response_part,
                )
                .await
            } else {
                fetch_response_without_cache(
                    ctx,
                    parent_objects,
                    subgraph_headers,
                    body,
                    plan.shape().id,
                    response_part,
                )
                .await
            }
        }
        .instrument(span)
        .await
    }
}

async fn fetch_response_without_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    parent_objects: ParentObjects,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    shape_id: RootFieldsShapeId,
    response_part: ResponsePartBuilder<'ctx>,
) -> ResponsePartBuilder<'ctx> {
    struct Ingester {
        parent_objects: ParentObjects,
        shape_id: RootFieldsShapeId,
    }

    impl ResponseIngester for Ingester {
        async fn ingest(
            self,
            result: Result<http::Response<Bytes>, GraphqlError>,
            mut response_part: ResponsePartBuilder<'_>,
        ) -> (Option<GraphqlResponseStatus>, ResponsePartBuilder<'_>) {
            let Self {
                shape_id,
                parent_objects,
            } = self;

            match result {
                Ok(http_response) => ingest_graphql_data(
                    response_part,
                    &parent_objects,
                    shape_id,
                    Deserializable::Json(http_response.body().as_ref()),
                ),
                Err(error) => {
                    response_part.insert_error_updates(&parent_objects, shape_id, error);
                    (None, response_part)
                }
            }
        }
    }

    execute_subgraph_request(
        ctx,
        subgraph_headers,
        body,
        response_part,
        Ingester {
            parent_objects,
            shape_id,
        },
    )
    .await
}

async fn fetch_response_with_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    parent_objects: ParentObjects,
    subgraph_headers: http::HeaderMap,
    body: Vec<u8>,
    shape_id: RootFieldsShapeId,
    response_part: ResponsePartBuilder<'ctx>,
) -> ResponsePartBuilder<'ctx> {
    match super::cache::fetch_response(ctx, &subgraph_headers, &body).await {
        Ok(ResponseCacheHit { data }) => {
            ctx.record_cache_hit();
            let (_, response_part) =
                ingest_graphql_data(response_part, &parent_objects, shape_id, Deserializable::Json(&data));
            response_part
        }
        Err(ResponseCacheMiss { key }) => {
            ctx.record_cache_miss();
            let ingester = GraphqlWithCachePutIngester {
                ctx: ctx.execution_context(),
                parent_objects,
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
    parent_objects: ParentObjects,
    shape_id: RootFieldsShapeId,
    subgraph_default_cache_ttl: Option<Duration>,
    cache_key: String,
}

impl<R> ResponseIngester for GraphqlWithCachePutIngester<'_, R>
where
    R: Runtime,
{
    async fn ingest(
        self,
        result: Result<http::Response<Bytes>, GraphqlError>,
        mut response_part: ResponsePartBuilder<'_>,
    ) -> (Option<GraphqlResponseStatus>, ResponsePartBuilder<'_>) {
        let Self {
            ctx,
            shape_id,
            parent_objects,
            subgraph_default_cache_ttl,
            cache_key,
        } = self;

        let http_response = match result {
            Ok(http_response) => http_response,
            Err(err) => {
                response_part.insert_error_updates(&parent_objects, shape_id, err);
                return (None, response_part);
            }
        };

        let (status, response_part) = ingest_graphql_data(
            response_part,
            &parent_objects,
            shape_id,
            Deserializable::Json(http_response.body().as_ref()),
        );

        if let Some(status) = status.filter(|s| s.is_success()) {
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

        (status, response_part)
    }
}

pub(super) fn ingest_graphql_data<'ctx, 'de>(
    response_part: ResponsePartBuilder<'ctx>,
    parent_objects: &ParentObjects,
    shape_id: RootFieldsShapeId,
    data: impl Into<Deserializable<'de>>,
) -> (Option<GraphqlResponseStatus>, ResponsePartBuilder<'ctx>) {
    debug_assert_eq!(parent_objects.len(), 1);
    let parent_object = parent_objects.iter().next().expect("Have at least one parent object");
    let state = response_part.into_seed_state(shape_id);
    let seed = GraphqlResponseSeed::new(
        state.parent_seed(parent_object),
        GraphqlErrorsSeed::new(&state, convert_root_error_path),
    );
    let status = match state.deserialize_data_with(data, seed) {
        Ok(status) => Some(status),
        Err(err) => {
            if let Some(error) = err {
                state.insert_error_update(parent_object, error);
            }
            None
        }
    };
    (status, state.into_response_part())
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
