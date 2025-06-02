mod with_cache;
mod without_cache;

use error::GraphqlError;
use grafbase_telemetry::{graphql::OperationType, span::subgraph::SubgraphRequestSpanBuilder};
use operation::OperationContext;
use schema::{GraphqlEndpointId, GraphqlFederationEntityResolverDefinition};
use serde_json::value::RawValue;
use tracing::Instrument;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult, RootFieldsShapeId},
    resolver::graphql::request::{SubgraphGraphqlRequest, SubgraphVariables},
    response::{ParentObjectId, ParentObjectSet, ParentObjects, ResponsePartBuilder},
};

use super::{
    SubgraphContext,
    request::{PreparedFederationEntityOperation, execute_subgraph_request},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FederationEntityResolver {
    pub endpoint_id: GraphqlEndpointId,
    pub shape_id: RootFieldsShapeId,
    pub subgraph_operation: PreparedFederationEntityOperation,
}

impl FederationEntityResolver {
    pub fn prepare(
        ctx: OperationContext<'_>,
        definition: GraphqlFederationEntityResolverDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        let subgraph_operation =
            PreparedFederationEntityOperation::build(ctx, plan_query_partition).map_err(|err| {
                tracing::error!("Failed to build query: {err}");
                PlanError::Internal
            })?;

        Ok(Self {
            endpoint_id: definition.endpoint().id,
            shape_id: plan_query_partition.shape_id(),
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
                operation_type: OperationType::Query.as_str(),
                sanitized_query: &self.subgraph_operation.query,
            },
        )
    }

    pub fn build_executor<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &SubgraphContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> FederationEntityExecutor<'ctx> {
        ctx.span().in_scope(|| {
            let extra_fields = vec![(
                "__typename".into(),
                serde_json::Value::String(plan.entity_definition().name().to_string()),
            )];
            let parent_objects_view = parent_objects.with_extra_constant_fields(&extra_fields);

            let mut entities_to_fetch = Vec::with_capacity(parent_objects.len());
            let mut entities_without_expected_requirements = Vec::new();

            for (id, object) in parent_objects_view.iter_with_id() {
                match serde_json::value::to_raw_value(&object) {
                    Ok(representation) => {
                        entities_to_fetch.push(EntityToFetch { id, representation });
                    }
                    Err(error) => {
                        entities_without_expected_requirements.push(EntityWithoutExpectedRequirements { id, error });
                    }
                }
            }

            FederationEntityExecutor {
                resolver: self,
                parent_objects: parent_objects.into_object_set(),
                response_part,
                entities_to_fetch,
                entities_without_expected_requirements,
            }
        })
    }
}

pub(super) struct EntityToFetch {
    pub id: ParentObjectId,
    pub representation: Box<RawValue>,
}

struct EntityWithoutExpectedRequirements {
    id: ParentObjectId,
    error: serde_json::Error,
}

pub(crate) struct FederationEntityExecutor<'ctx> {
    resolver: &'ctx FederationEntityResolver,
    parent_objects: ParentObjectSet,
    response_part: ResponsePartBuilder<'ctx>,
    entities_to_fetch: Vec<EntityToFetch>,
    entities_without_expected_requirements: Vec<EntityWithoutExpectedRequirements>,
}

impl<'ctx> FederationEntityExecutor<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: &mut SubgraphContext<'ctx, R>) -> ResponsePartBuilder<'ctx> {
        let Self {
            resolver:
                FederationEntityResolver {
                    subgraph_operation,
                    shape_id,
                    ..
                },
            parent_objects,
            mut response_part,
            entities_to_fetch,
            entities_without_expected_requirements,
        } = self;
        let span = ctx.span();

        async move {
            let subgraph_headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            for EntityWithoutExpectedRequirements { id, error } in entities_without_expected_requirements {
                tracing::error!("Could not retrieve entity because of missing requirements: {error}");
                // Not really sure if that's really the right logic. In the federation-audit
                // `null-keys` test no errors are expected here when an entity could not be
                // retrieved.
                response_part.insert_empty_update(&parent_objects[id], *shape_id);
            }

            if entities_to_fetch.is_empty() {
                return response_part;
            }

            if ctx.endpoint().config.cache_ttl.is_some() {
                fetch_entities_with_cache(
                    ctx,
                    parent_objects,
                    subgraph_headers,
                    subgraph_operation,
                    entities_to_fetch,
                    *shape_id,
                    response_part,
                )
                .await
            } else {
                fetch_entities_without_cache(
                    ctx,
                    parent_objects,
                    subgraph_headers,
                    subgraph_operation,
                    entities_to_fetch,
                    *shape_id,
                    response_part,
                )
                .await
            }
        }
        .instrument(span)
        .await
    }
}

pub(super) struct RepresentationListView<I>(I);

impl<'a, I> serde::Serialize for RepresentationListView<I>
where
    I: Clone + IntoIterator<Item = &'a RawValue>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.clone())
    }
}

pub(super) async fn fetch_entities_without_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    parent_objects: ParentObjectSet,
    subgraph_headers: http::HeaderMap,
    subgraph_operation: &PreparedFederationEntityOperation,
    entities_to_fetch: Vec<EntityToFetch>,
    shape_id: RootFieldsShapeId,
    mut response_part: ResponsePartBuilder<'ctx>,
) -> ResponsePartBuilder<'ctx> {
    let variables = SubgraphVariables {
        ctx: ctx.input_value_context(),
        variables: &subgraph_operation.variables,
        extra_variables: vec![(
            &subgraph_operation.entities_variable_name,
            RepresentationListView(entities_to_fetch.iter().map(|entity| entity.representation.as_ref())),
        )],
    };

    tracing::debug!(
        "Executing request to subgraph named '{}' with query and variables:\n{}\n{}",
        ctx.endpoint().subgraph_name(),
        subgraph_operation.query,
        serde_json::to_string_pretty(&variables).unwrap_or_default()
    );

    // We use RawValue underneath, so can't use sonic_rs. RawValue doesn't do any copies
    // compared to sonic_rs::LazyValue
    let body = match serde_json::to_vec(&SubgraphGraphqlRequest {
        query: &subgraph_operation.query,
        variables,
    }) {
        Ok(body) => body,
        Err(err) => {
            tracing::error!("Failed to serialize query: {err}");
            response_part.insert_error_updates(&parent_objects, shape_id, GraphqlError::internal_server_error());
            return response_part;
        }
    };

    let ingester = without_cache::EntityIngester {
        shape_id,
        parent_objects,
        fetched_entities: entities_to_fetch,
    };

    execute_subgraph_request(ctx, subgraph_headers, body, response_part, ingester).await
}

pub(super) async fn fetch_entities_with_cache<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    parent_objects: ParentObjectSet,
    subgraph_headers: http::HeaderMap,
    subgraph_operation: &PreparedFederationEntityOperation,
    entities_to_fetch: Vec<EntityToFetch>,
    shape_id: RootFieldsShapeId,
    mut response_part: ResponsePartBuilder<'ctx>,
) -> ResponsePartBuilder<'ctx> {
    let cache_fetch_outcome = super::cache::fetch_entities(ctx, &subgraph_headers, entities_to_fetch).await;
    if cache_fetch_outcome.misses.is_empty() {
        ctx.record_cache_hit();
        let state = response_part.into_seed_state(shape_id);
        with_cache::ingest_hits(&state, &parent_objects, cache_fetch_outcome.hits);
        return state.into_response_part();
    } else if cache_fetch_outcome.hits.is_empty() {
        ctx.record_cache_miss();
    } else {
        ctx.record_cache_partial_hit();
    }

    let variables = SubgraphVariables {
        ctx: ctx.input_value_context(),
        variables: &subgraph_operation.variables,
        extra_variables: vec![(
            &subgraph_operation.entities_variable_name,
            RepresentationListView(
                cache_fetch_outcome
                    .misses
                    .iter()
                    .map(|miss| miss.representation.as_ref()),
            ),
        )],
    };

    tracing::debug!(
        "Executing request to subgraph named '{}' with query and variables:\n{}\n{}",
        ctx.endpoint().subgraph_name(),
        subgraph_operation.query,
        serde_json::to_string_pretty(&variables).unwrap_or_default()
    );

    // We use RawValue underneath, so can't use sonic_rs. RwaValue doesn't do any copies
    // compared to sonic_rs::LazyValue
    let body = match serde_json::to_vec(&SubgraphGraphqlRequest {
        query: &subgraph_operation.query,
        variables,
    }) {
        Ok(body) => body,
        Err(err) => {
            tracing::error!("Failed to serialize query: {err}");
            response_part.insert_error_updates(&parent_objects, shape_id, GraphqlError::internal_server_error());
            return response_part;
        }
    };

    let ingester = with_cache::PartiallyCachedEntitiesIngester {
        ctx: ctx.execution_context(),
        parent_objects,
        cache_fetch_outcome,
        shape_id,
        subgraph_default_cache_ttl: ctx.endpoint().config.cache_ttl,
    };

    execute_subgraph_request(ctx, subgraph_headers, body, response_part, ingester).await
}
