use bytes::Bytes;
use futures::future::join_all;
use grafbase_telemetry::{gql_response_status::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpan};
use http::HeaderMap;
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{FederationEntityResolveDefinitionrWalker, GraphqlEndpointId};
use serde::{de::DeserializeSeed, Deserialize};
use serde_json::value::RawValue;
use std::{borrow::Cow, future::Future, time::Duration};
use tracing::Instrument;

use crate::{
    execution::{ExecutionContext, ExecutionError, PlanningResult},
    operation::{OperationType, PlanWalker},
    response::{ResponseObjectsView, SubgraphResponse},
    sources::{
        graphql::{
            deserialize::{EntitiesErrorsSeed, GraphqlResponseSeed},
            request::{SubgraphGraphqlRequest, SubgraphVariables},
        },
        ExecutionResult, Resolver,
    },
    Runtime,
};

use super::{
    deserialize::EntitiesDataSeed,
    request::{execute_subgraph_request, PreparedFederationEntityOperation, ResponseIngester},
};

pub(crate) struct FederationEntityResolver {
    endpoint_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityResolver {
    pub fn prepare(
        definition: FederationEntityResolveDefinitionrWalker<'_>,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Resolver> {
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(Resolver::FederationEntity(Self {
            endpoint_id: definition.endpoint().id(),
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

        let endpoint = ctx.engine.schema.walk(self.endpoint_id);
        let span = SubgraphRequestSpan {
            name: endpoint.subgraph_name(),
            operation_type: OperationType::Query.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: endpoint.url(),
        }
        .into_span();

        let cache_ttl = endpoint.entity_cache_ttl();

        let fut = {
            let span = span.clone();
            async move {
                let mut ingester = EntityIngester {
                    ctx,
                    cache_entries: None,
                    subgraph_response,
                    cache_ttl,
                };

                let headers = ctx.subgraph_headers_with_rules(endpoint.header_rules());

                if cache_ttl.is_some() {
                    match cache_fetches(ctx, endpoint, &headers, representations).await {
                        CacheFetchOutcome::FullyCached { cache_entries } => {
                            ingester.cache_entries = Some(cache_entries);

                            let (_, response) = ingester
                                .ingest(Bytes::from_static(br#"{"data": {"_entities": []}}"#))
                                .await?;

                            return Ok(response);
                        }
                        CacheFetchOutcome::Other {
                            cache_entries,
                            filtered_representations,
                        } => {
                            ingester.cache_entries = cache_entries;
                            representations = filtered_representations;
                        }
                    }
                }
                let variables = SubgraphVariables {
                    plan,
                    variables: &self.operation.variables,
                    extra_variables: vec![(&self.operation.entities_variable_name, representations)],
                };

                tracing::debug!(
                    "Query {}\n{}\n{}",
                    endpoint.subgraph_name(),
                    self.operation.query,
                    serde_json::to_string_pretty(&variables).unwrap_or_default()
                );
                let body = serde_json::to_vec(&SubgraphGraphqlRequest {
                    query: &self.operation.query,
                    variables,
                })
                .map_err(|err| format!("Failed to serialize query: {err}"))?;

                let retry_budget = ctx.engine.get_retry_budget_for_query(self.endpoint_id);

                execute_subgraph_request(
                    ctx,
                    span.clone(),
                    self.endpoint_id,
                    retry_budget,
                    move || FetchRequest {
                        url: endpoint.url(),
                        headers,
                        json_body: Bytes::from(body),
                        timeout: endpoint.timeout(),
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

async fn cache_fetches<'ctx, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: schema::SchemaWalker<'_, GraphqlEndpointId>,
    headers: &http::HeaderMap,
    representations: Vec<Box<RawValue>>,
) -> CacheFetchOutcome {
    let fetches = representations
        .iter()
        .map(|repr| cache_fetch(ctx, endpoint.subgraph_name(), headers, repr));

    let cache_entries = join_all(fetches).await;
    let fully_cached = !cache_entries.iter().any(CacheEntry::is_miss);

    if fully_cached {
        return CacheFetchOutcome::FullyCached { cache_entries };
    }

    let filtered_representations = representations
        .into_iter()
        .zip(cache_entries.iter())
        .filter(|(_, cache_entry)| cache_entry.is_miss())
        .map(|(repr, _)| repr)
        .collect();

    CacheFetchOutcome::Other {
        cache_entries: Some(cache_entries),
        filtered_representations,
    }
}

enum CacheFetchOutcome {
    FullyCached {
        cache_entries: Vec<CacheEntry>,
    },
    Other {
        cache_entries: Option<Vec<CacheEntry>>,
        filtered_representations: Vec<Box<RawValue>>,
    },
}

struct EntityIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    cache_entries: Option<Vec<CacheEntry>>,
    subgraph_response: SubgraphResponse,
    cache_ttl: Option<Duration>,
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
            cache_entries,
            mut subgraph_response,
            cache_ttl,
        } = self;

        let status = {
            let response = subgraph_response.as_mut();
            GraphqlResponseSeed::new(
                EntitiesDataSeed {
                    ctx,
                    response: response.clone(),
                    cache_entries: cache_entries.as_deref(),
                },
                EntitiesErrorsSeed::new(ctx, response),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?
        };

        if let Some(cache_ttl) = cache_ttl {
            if let Some(cache_entries) = cache_entries.filter(|_| status.is_success()) {
                update_cache(ctx, cache_ttl, bytes, cache_entries).await
            }
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
                .entity_cache()
                .put(&key, Cow::Borrowed(bytes), cache_ttl)
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

async fn cache_fetch<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    subgraph_name: &str,
    headers: &HeaderMap,
    repr: &RawValue,
) -> CacheEntry {
    let key = build_cache_key(subgraph_name, headers, repr);

    let data = ctx
        .engine
        .runtime
        .entity_cache()
        .get(&key)
        .await
        .inspect_err(|err| tracing::warn!("Failed to read the cache key {key}: {err}"))
        .ok()
        .flatten();

    match data {
        Some(data) => CacheEntry::Hit { data },
        None => CacheEntry::Miss { key },
    }
}

fn build_cache_key(subgraph_name: &str, headers: &HeaderMap, repr: &RawValue) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(subgraph_name.as_bytes());
    hasher.update(&headers.len().to_le_bytes());
    for (name, value) in headers {
        hasher.update(&name.as_str().len().to_le_bytes());
        hasher.update(name.as_str().as_bytes());
        hasher.update(&value.len().to_le_bytes());
        hasher.update(value.as_bytes());
    }
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
