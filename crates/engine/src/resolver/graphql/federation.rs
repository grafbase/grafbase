mod with_cache;
mod without_cache;

use grafbase_telemetry::{graphql::OperationType, span::subgraph::SubgraphRequestSpanBuilder};
use schema::{GraphqlEndpointId, GraphqlFederationEntityResolverDefinition};
use serde_json::value::RawValue;
use tracing::Instrument;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult},
    resolver::{
        ExecutionResult, Resolver,
        graphql::request::{SubgraphGraphqlRequest, SubgraphVariables},
    },
    response::{InputObjectId, ObjectUpdate, ResponseObjectsView, SubgraphResponse},
};

use super::{
    SubgraphContext,
    request::{PreparedFederationEntityOperation, execute_subgraph_request},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FederationEntityResolver {
    pub endpoint_id: GraphqlEndpointId,
    pub subgraph_operation: PreparedFederationEntityOperation,
}

impl FederationEntityResolver {
    pub fn prepare(
        definition: GraphqlFederationEntityResolverDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Resolver> {
        let subgraph_operation = PreparedFederationEntityOperation::build(plan_query_partition).map_err(|err| {
            tracing::error!("Failed to build query: {err}");
            PlanError::InternalError
        })?;

        Ok(Resolver::FederationEntity(Self {
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
                operation_type: OperationType::Query.as_str(),
                sanitized_query: &self.subgraph_operation.query,
            },
        )
    }

    pub fn prepare_request<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &SubgraphContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<FederationEntityRequest<'ctx>> {
        ctx.span().in_scope(|| {
            let extra_fields = vec![(
                "__typename".into(),
                serde_json::Value::String(plan.entity_definition().name().to_string()),
            )];
            let root_response_objects = root_response_objects.with_extra_constant_fields(&extra_fields);

            let mut entities_to_fetch = Vec::with_capacity(root_response_objects.len());
            let mut entities_without_expected_requirements = Vec::new();

            for (id, object) in root_response_objects.iter_with_id() {
                match serde_json::value::to_raw_value(&object) {
                    Ok(representation) => {
                        entities_to_fetch.push(EntityToFetch { id, representation });
                    }
                    Err(error) => {
                        entities_without_expected_requirements.push(EntityWithoutExpectedRequirements { id, error });
                    }
                }
            }

            Ok(FederationEntityRequest {
                resolver: self,
                subgraph_response,
                entities_to_fetch,
                entities_without_expected_requirements,
            })
        })
    }
}

pub(super) struct EntityToFetch {
    pub id: InputObjectId,
    pub representation: Box<RawValue>,
}

struct EntityWithoutExpectedRequirements {
    id: InputObjectId,
    error: serde_json::Error,
}

pub(crate) struct FederationEntityRequest<'ctx> {
    resolver: &'ctx FederationEntityResolver,
    subgraph_response: SubgraphResponse,
    entities_to_fetch: Vec<EntityToFetch>,
    entities_without_expected_requirements: Vec<EntityWithoutExpectedRequirements>,
}

impl<'ctx> FederationEntityRequest<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: &mut SubgraphContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            resolver: FederationEntityResolver { subgraph_operation, .. },
            mut subgraph_response,
            entities_to_fetch,
            entities_without_expected_requirements,
        } = self;
        let span = ctx.span();

        async move {
            let subgraph_headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());

            for EntityWithoutExpectedRequirements { id, error } in entities_without_expected_requirements {
                tracing::error!("Could not retrieve entity because of missing requirements: {error}");
                subgraph_response.insert_update(
                    id,
                    // Not really sure if that's really the right logic. In the federation-audit
                    // `null-keys` test no errors are expected here when an entity could not be
                    // retrieved.
                    ObjectUpdate::PropagateNullWithoutError,
                )
            }

            if entities_to_fetch.is_empty() {
                return Ok(subgraph_response);
            }

            if ctx.endpoint().config.cache_ttl.is_some() {
                fetch_entities_with_cache(
                    ctx,
                    subgraph_headers,
                    subgraph_operation,
                    entities_to_fetch,
                    subgraph_response,
                )
                .await
            } else {
                fetch_entities_without_cache(
                    ctx,
                    subgraph_headers,
                    subgraph_operation,
                    entities_to_fetch,
                    subgraph_response,
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

pub(super) async fn fetch_entities_without_cache<R: Runtime>(
    ctx: &mut SubgraphContext<'_, R>,
    subgraph_headers: http::HeaderMap,
    subgraph_operation: &PreparedFederationEntityOperation,
    entities_to_fetch: Vec<EntityToFetch>,
    subgraph_response: SubgraphResponse,
) -> ExecutionResult<SubgraphResponse> {
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

    let body = serde_json::to_vec(&SubgraphGraphqlRequest {
        query: &subgraph_operation.query,
        variables,
    })
    .map_err(|err| format!("Failed to serialize query: {err}"))?;

    let ingester = without_cache::EntityIngester {
        ctx: ctx.execution_context(),
        subgraph_response,
        fetched_entities: entities_to_fetch,
    };

    execute_subgraph_request(ctx, subgraph_headers, body, ingester).await
}

pub(super) async fn fetch_entities_with_cache<R: Runtime>(
    ctx: &mut SubgraphContext<'_, R>,
    subgraph_headers: http::HeaderMap,
    subgraph_operation: &PreparedFederationEntityOperation,
    entities_to_fetch: Vec<EntityToFetch>,
    subgraph_response: SubgraphResponse,
) -> ExecutionResult<SubgraphResponse> {
    let cache_fetch_outcome = super::cache::fetch_entities(ctx, &subgraph_headers, entities_to_fetch).await;
    if cache_fetch_outcome.misses.is_empty() {
        ctx.record_cache_hit();
        return with_cache::ingest_hits(ctx.execution_context(), cache_fetch_outcome.hits, subgraph_response);
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

    let body = serde_json::to_vec(&SubgraphGraphqlRequest {
        query: &subgraph_operation.query,
        variables,
    })
    .map_err(|err| format!("Failed to serialize query: {err}"))?;

    let ingester = with_cache::PartiallyCachedEntitiesIngester {
        ctx: ctx.execution_context(),
        cache_fetch_outcome,
        subgraph_response,
        subgraph_default_cache_ttl: ctx.endpoint().config.cache_ttl,
    };

    execute_subgraph_request(ctx, subgraph_headers, body, ingester).await
}
