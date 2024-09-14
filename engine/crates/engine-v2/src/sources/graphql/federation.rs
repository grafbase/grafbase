use bytes::Bytes;
use futures::future::join_all;
use grafbase_telemetry::{graphql::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpanBuilder};
use http::HeaderMap;
use runtime::bytes::OwnedOrSharedBytes;
use schema::{GraphqlEndpoint, GraphqlEndpointId, GraphqlFederationEntityResolverDefinition};
use serde::{de::DeserializeSeed, Deserialize};
use serde_json::value::RawValue;
use std::{borrow::Cow, time::Duration};
use tracing::Instrument;
use walker::Walk;

use crate::{
    execution::{ExecutionContext, ExecutionError, PlanningResult},
    operation::{CacheScope, OperationType, PlanWalker},
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
    calculate_cache_ttl,
    deserialize::EntitiesDataSeed,
    request::{execute_subgraph_request, PreparedFederationEntityOperation, ResponseIngester},
    SubgraphContext,
};

pub(crate) struct FederationEntityResolver {
    endpoint_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityResolver {
    /// Prepares a `FederationEntityResolver` from the provided definition and plan.
    ///
    /// This function builds the underlying GraphQL operation for the federation entity.
    /// It captures any errors that occur during the building process, returning a
    /// `PlanningResult` that can either yield a `Resolver` or an error message.
    pub fn prepare(
        definition: GraphqlFederationEntityResolverDefinition<'_>,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Resolver> {
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;

        Ok(Resolver::FederationEntity(Self {
            endpoint_id: definition.endpoint().id(),
            operation,
        }))
    }

    /// Builds a `SubgraphContext` using the provided `ExecutionContext`.
    ///
    /// This function constructs a new context specifically for the subgraph,
    /// utilizing the endpoint associated with the federation entity resolver.
    ///
    /// # Parameters
    ///
    /// - `ctx`: The execution context containing the runtime and other execution-related details.
    ///
    /// # Returns
    ///
    /// A `SubgraphContext` that holds the necessary information for making requests
    /// to the specified subgraph and state data tracking the request execution.
    pub fn build_subgraph_context<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> SubgraphContext<'ctx, R> {
        let endpoint = self.endpoint_id.walk(ctx.schema());
        SubgraphContext::new(
            ctx,
            endpoint,
            SubgraphRequestSpanBuilder {
                subgraph_name: endpoint.subgraph_name(),
                operation_type: OperationType::Query.as_str(),
                sanitized_query: &self.operation.query,
            },
        )
    }

    #[tracing::instrument(skip_all)]
    /// Prepares a request for the federation entity.
    ///
    /// This function constructs a `FederationEntityRequest` which encapsulates the necessary
    /// information required to execute a request against the specified subgraph. It takes in
    /// various parameters including the execution context, plan, root response objects, and
    /// the initial subgraph response.
    ///
    /// # Parameters
    ///
    /// - `ctx`: The context for the subgraph, providing access to the execution context and
    ///   associated endpoint.
    /// - `plan`: A `PlanWalker` describing the plan for the current execution.
    /// - `root_response_objects`: The root response objects added to the subgraph query.
    /// - `subgraph_response`: The initial response from the subgraph which is modified during
    ///   processing.
    ///
    /// # Returns
    ///
    /// Returns an `ExecutionResult` containing a `FederationEntityRequest`, which can be
    /// executed to obtain data from the subgraph. In case of an error, it will contain the
    /// relevant error information.
    pub fn prepare_request<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &SubgraphContext<'ctx, R>,
        plan: PlanWalker<'ctx, ()>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<FederationEntityRequest<'ctx>> {
        ctx.span().in_scope(|| {
            let root_response_objects = root_response_objects.with_extra_constant_fields(vec![(
                "__typename".to_string(),
                serde_json::Value::String(entity_name(ctx, plan)),
            )]);

            let representations = root_response_objects
                .iter()
                .map(|object| serde_json::value::to_raw_value(&object))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(FederationEntityRequest {
                resolver: self,
                plan,
                subgraph_response,
                representations,
            })
        })
    }
}

pub(crate) struct FederationEntityRequest<'ctx> {
    resolver: &'ctx FederationEntityResolver,
    plan: PlanWalker<'ctx>,
    subgraph_response: SubgraphResponse,
    representations: Vec<Box<RawValue>>,
}

impl<'ctx> FederationEntityRequest<'ctx> {
    /// Executes the federation entity request against the subgraph.
    ///
    /// This function sends a request to the specified subgraph using the
    /// representations provided during the preparation of the request. It retrieves the
    /// associated data from the subgraph and returns the processed response.
    ///
    /// # Type Parameters
    ///
    /// - `'ctx`: The lifetime of the request context.
    /// - `R`: The execution runtime.
    ///
    /// # Parameters
    ///
    /// - `ctx`: A mutable reference to the `SubgraphContext` which contains the execution
    ///   context and related information needed for making the request.
    ///
    /// # Returns
    ///
    /// Returns an `ExecutionResult` containing the `SubgraphResponse` upon successful
    /// execution of the request. In case of any errors during the execution, the result will
    /// contain the corresponding error information.
    pub async fn execute<R: Runtime>(self, ctx: &mut SubgraphContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            resolver: FederationEntityResolver { operation, .. },
            plan,
            subgraph_response,
            mut representations,
        } = self;
        let span = ctx.span();

        async move {
            let cache_ttl = ctx.endpoint().config.cache_ttl;
            let mut ingester = EntityIngester {
                ctx: ctx.execution_context(),
                cache_entries: None,
                subgraph_response,
                cache_ttl,
            };

            let headers = ctx.subgraph_headers_with_rules(ctx.endpoint().header_rules());
            let additional_scopes = plan
                .cache_scopes()
                .map(|scope| match scope {
                    CacheScope::Authenticated => "authenticated".into(),
                    CacheScope::RequiresScopes(scopes) => {
                        let mut hasher = blake3::Hasher::new();
                        hasher.update(b"requiresScopes");
                        hasher.update(&scopes.scopes().len().to_le_bytes());
                        for scope in scopes.scopes() {
                            hasher.update(&scope.len().to_le_bytes());
                            hasher.update(scope.as_bytes());
                        }
                        hasher.finalize().to_hex().to_string()
                    }
                })
                .collect::<Vec<_>>();

            if cache_ttl.is_some() {
                match cache_fetches(ctx, &headers, representations, &additional_scopes).await {
                    CacheFetchOutcome::FullyCached { cache_entries } => {
                        ctx.record_cache_hit();
                        ingester.cache_entries = Some(cache_entries);

                        let (_, response) = ingester
                            .ingest(http::Response::new(
                                Bytes::from_static(br#"{"data": {"_entities": []}}"#).into(),
                            ))
                            .await?;

                        return Ok(response);
                    }
                    CacheFetchOutcome::Other {
                        cache_entries,
                        filtered_representations,
                    } => {
                        if cache_entries
                            .as_ref()
                            .map(|entries| entries.iter().any(|e| e.is_hit()))
                            .unwrap_or(true)
                        {
                            ctx.record_cache_partial_hit();
                        } else {
                            ctx.record_cache_miss();
                        }

                        ingester.cache_entries = cache_entries;
                        representations = filtered_representations;
                    }
                }
            }

            let variables = SubgraphVariables {
                plan,
                variables: &operation.variables,
                extra_variables: vec![(&operation.entities_variable_name, representations)],
            };

            tracing::debug!(
                "Executing request to subgraph named '{}' with query and variables:\n{}\n{}",
                ctx.endpoint().subgraph_name(),
                self.resolver.operation.query,
                serde_json::to_string_pretty(&variables).unwrap_or_default()
            );

            let body = serde_json::to_vec(&SubgraphGraphqlRequest {
                query: &operation.query,
                variables,
            })
            .map_err(|err| format!("Failed to serialize query: {err}"))?;

            execute_subgraph_request(ctx, headers, Bytes::from(body), ingester).await
        }
        .instrument(span)
        .await
    }
}

/// Fetches cache entries for the given representations.
///
/// This function attempts to retrieve cached entries for a list of representations from the
/// given subgraph context, headers, and additional scopes.
///
/// # Type Parameters
///
/// - `'ctx`: The lifetime of the request context.
/// - `R`: The execution runtime.
///
/// # Parameters
///
/// - `ctx`: A mutable reference to the `SubgraphContext` containing the execution context.
/// - `headers`: A reference to the HTTP headers used in the request.
/// - `representations`: A vector of representations for which cache entries are requested.
/// - `additional_scopes`: An array of additional cache scopes to consider.
///
/// # Returns
///
/// Returns a `CacheFetchOutcome`, indicating whether all requested cache entries were
/// fully cached, or if some were missing, along with the relevant cache entries.
async fn cache_fetches<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    headers: &http::HeaderMap,
    representations: Vec<Box<RawValue>>,
    additional_scopes: &[String],
) -> CacheFetchOutcome {
    let fetches = representations
        .iter()
        .map(|repr| cache_fetch(ctx, ctx.endpoint, headers, repr, additional_scopes));

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
    /// Indicates that all requested cache entries were successfully retrieved from cache.
    FullyCached {
        /// A vector containing the cache entries that were retrieved.
        cache_entries: Vec<CacheEntry>,
    },
    /// Represents a scenario where not all cache entries were available.
    Other {
        /// An optional vector of cache entries that were retrieved, if any.
        cache_entries: Option<Vec<CacheEntry>>,
        /// A vector of representations for which cache entries were not found.
        filtered_representations: Vec<Box<RawValue>>,
    },
}

struct EntityIngester<'ctx, R: Runtime> {
    /// The execution context for the federation entity.
    ///
    /// This provides access to runtime execution and schema related operations.
    ctx: ExecutionContext<'ctx, R>,

    /// An optional vector of cache entries.
    ///
    /// This holds the cache entries that may have been retrieved during processing.
    cache_entries: Option<Vec<CacheEntry>>,

    /// The response from the subgraph.
    ///
    /// This contains the data returned by the subgraph for the entity request.
    subgraph_response: SubgraphResponse,

    /// An optional time-to-live for the cache.
    ///
    /// This specifies the duration for which cache entries are considered valid.
    cache_ttl: Option<Duration>,
}

pub enum CacheEntry {
    /// Represents a cache miss scenario, containing the key that was not found.
    Miss { key: String },

    /// Represents a cache hit scenario, containing the data retrieved from the cache.
    Hit { data: Vec<u8> },
}

impl CacheEntry {
    /// Determines whether the cache entry represents a miss scenario.
    ///
    /// A cache miss occurs when the requested data could not be found in the cache.
    ///
    /// # Returns
    ///
    /// Returns `true` if the cache entry is a miss; otherwise, it returns `false`.
    pub fn is_miss(&self) -> bool {
        matches!(self, CacheEntry::Miss { .. })
    }

    /// Determines whether the cache entry represents a hit scenario.
    ///
    /// A cache hit occurs when the requested data was successfully found in the cache.
    ///
    /// # Returns
    ///
    /// Returns `true` if the cache entry is a hit; otherwise, it returns `false`.
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheEntry::Hit { .. })
    }

    /// Returns the underlying data of the cache entry if it's a hit.
    ///
    /// This method provides access to the cached data in cases where the entry
    /// was successfully retrieved from the cache. If the entry represents a miss,
    /// it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the data as a byte slice if the cache entry
    /// is a hit, or `None` if it is a miss.
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
    async fn ingest(
        self,
        http_response: http::Response<OwnedOrSharedBytes>,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
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
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))?
        };

        let cache_ttl = calculate_cache_ttl(status, http_response.headers(), cache_ttl);

        if let Some((cache_ttl, cache_entries)) = cache_ttl.zip(cache_entries) {
            update_cache(ctx, cache_ttl, http_response.into_body(), cache_entries).await
        }

        Ok((status, subgraph_response))
    }
}

/// Updates the cache with entities retrieved from the subgraph response.
///
/// This function iterates over cache entries and attempts to update the
/// cache with corresponding entity data. If a cache entry represents a miss,
/// it retrieves the appropriate entity from the response and caches it with
/// the specified time-to-live (TTL).
///
/// # Type Parameters
///
/// - `R`: The execution runtime used during the caching process.
///
/// # Parameters
///
/// - `ctx`: The execution context providing access to runtime operations.
/// - `cache_ttl`: The duration for which the cache entries are valid.
/// - `bytes`: The raw bytes of the response body to deserialize entities from.
/// - `cache_entries`: A vector of cache entries to check and update.
async fn update_cache<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    cache_ttl: Duration,
    bytes: OwnedOrSharedBytes,
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

/// Fetches a cache entry for a given representation.
///
/// This function attempts to retrieve a cached entity based on the provided representation's
/// details, including the subgraph name, headers, and additional scopes.
///
/// # Parameters
///
/// - `ctx`: The execution context containing the runtime operations.
/// - `endpoint`: The endpoint of the GraphQL subgraph being queried.
/// - `headers`: The HTTP headers associated with the request.
/// - `repr`: A raw value representing the entity to be fetched from the cache.
/// - `additional_scopes`: A list of additional cache scopes to consider during the fetch.
///
/// # Returns
///
/// Returns a `CacheEntry`, indicating whether the requested entity was found in the cache,
/// and, if so, the associated data, or a cache miss indicating the key associated with the
/// request.
async fn cache_fetch<'ctx, R: Runtime>(
    ctx: &ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpoint<'ctx>,
    headers: &HeaderMap,
    repr: &RawValue,
    additional_scopes: &[String],
) -> CacheEntry {
    let key = build_cache_key(endpoint.subgraph_name(), headers, repr, additional_scopes);

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

/// Constructs a cache key for the given representation based on the provided parameters.
///
/// This function generates a unique string key that combines the subgraph name, HTTP headers,
/// the raw representation of the entity, and any additional scopes. This key is used for
/// accessing cached entries related to the specified entity in the subgraph.
///
/// # Parameters
///
/// - `subgraph_name`: The name of the subgraph to which the entity belongs.
/// - `headers`: The HTTP headers associated with the request, which may affect caching behavior.
/// - `repr`: A raw value representing the entity whose cache entry is being built.
/// - `additional_scopes`: A list of additional scopes that can influence cache access.
///
/// # Returns
///
/// A `String` containing the generated cache key, which uniquely identifies the cached entry.
fn build_cache_key(subgraph_name: &str, headers: &HeaderMap, repr: &RawValue, additional_scopes: &[String]) -> String {
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
    hasher.update(&additional_scopes.len().to_le_bytes());
    for scope in additional_scopes {
        hasher.update(&scope.len().to_le_bytes());
        hasher.update(scope.as_bytes());
    }
    hasher.update(repr.get().as_bytes());
    hasher.finalize().to_string()
}

fn entity_name<R: Runtime>(ctx: &ExecutionContext<'_, R>, plan: PlanWalker<'_, ()>) -> String {
    ctx.engine
        .schema
        .walk(plan.logical_plan().as_ref().entity_id)
        .name()
        .to_string()
}
