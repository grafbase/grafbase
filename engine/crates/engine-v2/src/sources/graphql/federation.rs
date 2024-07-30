use bytes::Bytes;
use futures::future::join_all;
use grafbase_telemetry::{gql_response_status::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpan};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{FederationEntityResolverWalker, GraphqlEndpointId};
use serde::{de::DeserializeSeed, Deserialize};
use serde_json::value::RawValue;
use std::{borrow::Cow, future::Future, time::Duration};
use tracing::Instrument;

use crate::{
    execution::{ExecutionContext, ExecutionError, PlanWalker, PlanningResult},
    operation::OperationType,
    response::{ResponseObjectsView, SubgraphResponse},
    sources::{
        graphql::deserialize::{EntitiesErrorsSeed, GraphqlResponseSeed},
        ExecutionResult, PreparedExecutor,
    },
    Runtime,
};

use super::{
    deserialize::EntitiesDataSeed,
    query::PreparedFederationEntityOperation,
    request::{execute_subgraph_request, ResponseIngester},
    variables::SubgraphVariables,
};

pub(crate) struct FederationEntityPreparedExecutor {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityPreparedExecutor {
    pub fn prepare(
        resolver: FederationEntityResolverWalker<'_>,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<PreparedExecutor> {
        let subgraph = resolver.endpoint();
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(PreparedExecutor::FederationEntity(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<impl Future<Output = ExecutionResult<SubgraphResponse>> + Send + 'fut>
    where
        'ctx: 'fut,
    {
        let root_response_objects = root_response_objects.with_extra_constant_fields(vec![(
            "__typename".to_string(),
            serde_json::Value::String(entity_name(ctx, plan)),
        )]);
        let mut representations = root_response_objects
            .iter()
            .map(|object| serde_json::to_string(&object).and_then(RawValue::from_string))
            .collect::<Result<Vec<_>, _>>()?;

        let subgraph = ctx.engine.schema.walk(self.subgraph_id);
        let span = SubgraphRequestSpan {
            name: subgraph.name(),
            operation_type: OperationType::Query.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: subgraph.url(),
        }
        .into_span();

        let cache_ttl = subgraph.entity_cache_ttl();

        let fut = {
            let span = span.clone();
            async move {
                let mut ingester = EntityIngester {
                    ctx,
                    plan,
                    cache_entries: None,
                    subgraph_response,
                    cache_ttl,
                };

                if ctx.engine.schema.settings.enable_entity_caching {
                    let fetches = representations
                        .iter()
                        .map(|repr| cache_fetch(ctx, subgraph.name(), repr));

                    let cache_entries = join_all(fetches).await;
                    let fully_cached = !cache_entries.iter().any(CacheEntry::is_miss);
                    ingester.cache_entries = Some(cache_entries);
                    if fully_cached {
                        let (_, response) = ingester
                            .ingest(Bytes::from_static(br#"{"data": {"_entities": []}}"#))
                            .await?;

                        return Ok(response);
                    }
                    representations = representations
                        .into_iter()
                        .zip(ingester.cache_entries.as_ref().unwrap())
                        .filter(|(_, cache_entry)| cache_entry.is_miss())
                        .map(|(repr, _)| repr)
                        .collect();
                }
                let variables = SubgraphVariables {
                    plan,
                    variables: &self.operation.variables,
                    inputs: vec![(&self.operation.entities_variable_name, representations)],
                };

                tracing::debug!(
                    "Query {}\n{}\n{}",
                    subgraph.name(),
                    self.operation.query,
                    serde_json::to_string_pretty(&variables).unwrap_or_default()
                );
                let json_body = serde_json::to_string(&serde_json::json!({
                    "query": self.operation.query,
                    "variables": variables
                }))
                .map_err(|err| format!("Failed to serialize query: {err}"))?;

                let retry_budget = ctx.engine.retry_budget_for_subgraph(self.subgraph_id);

                execute_subgraph_request(
                    ctx,
                    span.clone(),
                    subgraph.name(),
                    move || FetchRequest {
                        url: subgraph.url(),
                        headers: ctx.subgraph_headers_with_rules(subgraph.header_rules()),
                        json_body,
                        subgraph_name: subgraph.name(),
                        timeout: subgraph.timeout(),
                        retry_budget,
                        rate_limiter: ctx.engine.runtime.rate_limiter(),
                    },
                    ingester,
                )
                .await
            }
        }
        .instrument(span);

        Ok(fut)
    }
}

struct EntityIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    plan: PlanWalker<'ctx, (), ()>,
    cache_entries: Option<Vec<CacheEntry>>,
    subgraph_response: SubgraphResponse,
    cache_ttl: Duration,
}

pub enum CacheEntry {
    Miss { key: String },
    Hit { data: Vec<u8> },
}

impl CacheEntry {
    pub fn is_miss(&self) -> bool {
        matches!(self, CacheEntry::Miss { .. })
    }

    pub fn as_data(&self) -> Option<&[u8]> {
        match self {
            CacheEntry::Hit { data } => Some(data),
            _ => None,
        }
    }
}

impl<'ctx, R> ResponseIngester for EntityIngester<'ctx, R>
where
    R: Runtime,
{
    async fn ingest(self, bytes: Bytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        let Self {
            ctx,
            plan,
            cache_entries,
            mut subgraph_response,
            cache_ttl,
        } = self;

        let status = {
            let response = subgraph_response.as_mut();
            GraphqlResponseSeed::new(
                EntitiesDataSeed {
                    response: response.clone(),
                    cache_entries: cache_entries.as_deref(),
                    plan,
                },
                EntitiesErrorsSeed {
                    response,
                    response_keys: plan.response_keys(),
                },
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?
        };

        if let Some(cache_entries) = cache_entries.filter(|_| status.is_success()) {
            update_cache(ctx, cache_ttl, bytes, cache_entries).await
        }

        Ok((status, subgraph_response))
    }
}

async fn update_cache<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    cache_ttl: Duration,
    bytes: Bytes,
    cache_entries: Vec<CacheEntry>,
) {
    let mut entities = match Response::deserialize(&mut serde_json::Deserializer::from_slice(&bytes)) {
        Ok(response) => response.data.entities.into_iter(),
        Err(err) => {
            tracing::warn!("Couldn't deserialize response for cache update: {err}");
            // This shouldn't really happen but if it does lets ignore it
            // Don't want cache stuff to break the actual request
            return;
        }
    };

    let mut update_futures = vec![];
    for entry in cache_entries {
        let CacheEntry::Miss { key } = entry else { continue };

        let Some(data) = entities.next() else {
            // This shouldn't really happen but if it does lets ignore it
            // Don't want cache stuff to break the actual request
            return;
        };
        let bytes = data.get().as_bytes();
        update_futures.push(async move {
            ctx.engine
                .runtime
                .kv()
                .put(&key, Cow::Borrowed(bytes), Some(cache_ttl))
                .await
                .inspect_err(|err| tracing::warn!("Failed to write the cache key {key}: {err}"))
                .ok();
        })
    }

    join_all(update_futures).await;
}

#[derive(serde::Deserialize)]
struct Response<'a> {
    #[serde(borrow)]
    data: Data<'a>,
}

#[derive(serde::Deserialize)]
struct Data<'a> {
    #[serde(borrow, rename = "_entities")]
    entities: Vec<&'a serde_json::value::RawValue>,
}

async fn cache_fetch<R: Runtime>(ctx: ExecutionContext<'_, R>, subgraph_name: &str, repr: &RawValue) -> CacheEntry {
    let key = build_cache_key(subgraph_name, repr);

    let data = ctx
        .engine
        .runtime
        .kv()
        .get(&key, Some(Duration::ZERO))
        .await
        .inspect_err(|err| tracing::warn!("Failed to read the cache key {key}: {err}"))
        .ok()
        .flatten();

    match data {
        Some(data) => CacheEntry::Hit { data },
        None => CacheEntry::Miss { key },
    }
}

fn build_cache_key(subgraph_name: &str, repr: &RawValue) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(subgraph_name.as_bytes());
    hasher.update(repr.get().as_bytes());
    hasher.finalize().to_string()
}

fn entity_name<R: Runtime>(ctx: ExecutionContext<'_, R>, plan: PlanWalker<'_, (), ()>) -> String {
    ctx.engine
        .schema
        .walker()
        .walk(schema::Definition::from(plan.logical_plan().as_ref().entity_id))
        .name()
        .to_string()
}
