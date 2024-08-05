use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use grafbase_telemetry::{gql_response_status::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpan};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{GraphqlEndpointId, RootFieldResolverDefinitionWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    request::{execute_subgraph_request, PreparedGraphqlOperation, ResponseIngester, SubgraphVariables},
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

    #[tracing::instrument(skip_all)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let endpoint = plan.schema().walk(self.endpoint_id);
        let variables = SubgraphVariables::<()> {
            plan,
            variables: &self.operation.variables,
            extra_variables: Vec::new(),
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

        let headers = ctx.subgraph_headers_with_rules(endpoint.header_rules());

        let cache_ttl_and_key = endpoint
            .entity_cache_ttl()
            .and_then(|ttl| Some((ttl, build_cache_key(endpoint.subgraph_name(), &body, &headers)?)));

        if let Some((_, cache_key)) = &cache_ttl_and_key {
            let cache_entry = ctx
                .engine
                .runtime
                .entity_cache()
                .get(cache_key)
                .await
                .inspect_err(|err| tracing::warn!("Failed to read the cache key {cache_key}: {err}"))
                .ok()
                .flatten();

            if let Some(bytes) = cache_entry {
                let response = subgraph_response.as_mut();

                GraphqlResponseSeed::new(
                    response.next_seed(ctx).ok_or("No object to update")?,
                    RootGraphqlErrors::new(ctx, response),
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

                return Ok(subgraph_response);
            };
        };

        let retry_budget = if self.operation.ty.is_mutation() {
            ctx.engine.get_retry_budget_for_mutation(self.endpoint_id)
        } else {
            ctx.engine.get_retry_budget_for_query(self.endpoint_id)
        };

        let span = SubgraphRequestSpan {
            name: endpoint.subgraph_name(),
            operation_type: self.operation.ty.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: endpoint.url(),
        }
        .into_span();

        execute_subgraph_request(
            ctx,
            span.clone(),
            self.endpoint_id,
            retry_budget,
            || FetchRequest {
                url: endpoint.url(),
                headers,
                json_body: Bytes::from(body),
                timeout: endpoint.timeout(),
            },
            GraphqlIngester {
                ctx,
                cache_ttl_and_key,
                subgraph_response,
            },
        )
        .instrument(span)
        .await
    }
}

fn build_cache_key(subgraph_name: &str, subgraph_request_body: &[u8], headers: &http::HeaderMap) -> Option<String> {
    let header_json = serde_json::to_vec(
        &headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_bytes()))
            .collect::<Vec<_>>(),
    )
    .inspect_err(|err| tracing::warn!("Couldn't encode headers as JSON for cache key: {err}"))
    .ok()?;

    let mut hasher = blake3::Hasher::new();
    hasher.update(subgraph_name.as_bytes());
    hasher.update(&header_json);
    hasher.update(subgraph_request_body);
    Some(hasher.finalize().to_string())
}

struct GraphqlIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph_response: SubgraphResponse,
    cache_ttl_and_key: Option<(Duration, String)>,
}

impl<'ctx, R> ResponseIngester for GraphqlIngester<'ctx, R>
where
    R: Runtime,
{
    async fn ingest(
        mut self,
        bytes: Bytes,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), crate::execution::ExecutionError> {
        let status = {
            let response = self.subgraph_response.as_mut();
            GraphqlResponseSeed::new(
                response.next_seed(self.ctx).ok_or("No object to update")?,
                RootGraphqlErrors::new(self.ctx, response),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?
        };

        if let Some((cache_ttl, cache_key)) = self.cache_ttl_and_key.filter(|_| status.is_success()) {
            // We could probably put this call into the background at some point, but for
            // simplicities sake I am not going to do that just now.
            self.ctx
                .engine
                .runtime
                .entity_cache()
                .put(&cache_key, Cow::Borrowed(bytes.as_ref()), cache_ttl)
                .await
                .inspect_err(|err| tracing::warn!("Failed to write the cache key {cache_key}: {err}"))
                .ok();
        }

        Ok((status, self.subgraph_response))
    }
}
