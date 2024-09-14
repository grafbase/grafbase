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
    execution::PlanningResult,
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
        let operation = PreparedGraphqlOperation::build(operation_type, plan)
            .map_err(|err| format!("Failed to build query: {err}"))?;

        Ok(Resolver::GraphQL(Self {
            endpoint_id: definition.endpoint().id(),
            operation,
        }))
    }

    /// Builds the subgraph context for the resolver.
    ///
    /// This method creates a new `SubgraphContext` using the provided execution context,
    /// endpoint information, and operation details. It sets up the context required for
    /// executing GraphQL requests against the subgraph.
    ///
    /// # Parameters
    ///
    /// - `ctx`: The execution context that carries state and configurations needed
    ///   throughout the lifecycle of the operation.
    ///
    /// # Returns
    ///
    /// Returns a `SubgraphContext` that carries state over this specific request.
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

    /// Executes the GraphQL operation against the subgraph.
    ///
    /// This asynchronous function prepares and sends a GraphQL request to the subgraph,
    /// processes the response, and handles caching.
    ///
    /// # Parameters
    ///
    /// - `ctx`: A mutable reference to the `SubgraphContext` that contains state
    ///   and configuration for the execution.
    /// - `plan`: A `PlanWalker` that represents the plan for executing the operation.
    /// - `subgraph_response`: A `SubgraphResponse` instance that holds the response
    ///   data from the subgraph.
    ///
    /// # Returns
    ///
    /// Returns an `ExecutionResult` which wraps the resulting `SubgraphResponse`.
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
            let cache_key = Some(build_cache_key(ctx.endpoint().subgraph_name(), &body, &headers));

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

                    let response = subgraph_response.as_mut();

                    GraphqlResponseSeed::new(
                        response.next_seed(ctx).ok_or("No object to update")?,
                        RootGraphqlErrors::new(ctx, response),
                    )
                    .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

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

/// Builds a unique cache key for the subgraph request.
///
/// This function takes the name of the subgraph, the body of the request, and the
/// headers, and generates a cache key using a hashing algorithm. The cache key can
/// be used to store and retrieve cached responses for the same request parameters.
///
/// # Parameters
///
/// - `subgraph_name`: The name of the subgraph being requested.
/// - `subgraph_request_body`: The serialized body of the GraphQL request as a byte slice.
/// - `headers`: The HTTP headers associated with the request.
///
/// # Returns
///
/// Returns an `Option<String>` containing the generated cache key, or `None` if
/// the key could not be generated.
fn build_cache_key(subgraph_name: &str, subgraph_request_body: &[u8], headers: &http::HeaderMap) -> String {
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
    hasher.finalize().to_string()
}

/// A structure responsible for ingesting GraphQL responses from the subgraph.
///
/// This struct manages the execution context, the response received from the subgraph,
/// and caching behavior based on the GraphQL response's status and provided cache settings.
struct GraphqlIngester<'ctx, R: Runtime> {
    /// The execution context for the GraphQL request, which holds state and configuration.
    ctx: ExecutionContext<'ctx, R>,

    /// The response from the subgraph that will be processed.
    subgraph_response: SubgraphResponse,

    /// Optional duration specifying the cache time-to-live.
    cache_ttl: Option<Duration>,

    /// Optional cache key used for storing and retrieving the response in the cache.
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
                response.next_seed(&self.ctx).ok_or("No object to update")?,
                RootGraphqlErrors::new(&self.ctx, response),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(http_response.body()))?
        };

        if let Some(cache_key) = self.cache_key {
            let cache_ttl = calculate_cache_ttl(status, http_response.headers(), self.cache_ttl);

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
