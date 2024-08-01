use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use grafbase_telemetry::{gql_response_status::GraphqlResponseStatus, span::subgraph::SubgraphRequestSpan};
use request::{execute_subgraph_request, ResponseIngester};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{GraphqlEndpointId, RootFieldResolverWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use self::query::PreparedGraphqlOperation;
use self::variables::SubgraphVariables;

use super::{ExecutionContext, ExecutionResult, PreparedExecutor};
use crate::{
    execution::{PlanWalker, PlanningResult},
    operation::OperationType,
    response::SubgraphResponse,
    sources::graphql::deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    Runtime,
};

mod deserialize;
mod federation;
mod query;
mod request;
mod subscription;
mod variables;

pub(crate) use federation::*;

pub(crate) struct GraphqlPreparedExecutor {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedGraphqlOperation,
}

impl GraphqlPreparedExecutor {
    pub fn prepare(
        resolver: RootFieldResolverWalker<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<PreparedExecutor> {
        let subgraph = resolver.endpoint();

        let operation = query::PreparedGraphqlOperation::build(operation_type, plan)
            .map_err(|err| format!("Failed to build query: {err}"))?;

        Ok(PreparedExecutor::GraphQL(Self {
            subgraph_id: subgraph.id(),
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
        let subgraph = plan.schema().walk(self.subgraph_id);

        let variables = SubgraphVariables::<()> {
            plan,
            variables: &self.operation.variables,
            inputs: Vec::new(),
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

        let span = SubgraphRequestSpan {
            name: subgraph.name(),
            operation_type: self.operation.ty.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: subgraph.url(),
        }
        .into_span();

        let cache_ttl_and_key = subgraph
            .entity_cache_ttl()
            .map(|ttl| (ttl, build_cache_key(&json_body)));

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
                    response.next_seed(plan).ok_or("No object to update")?,
                    RootGraphqlErrors {
                        response,
                        response_keys: plan.response_keys(),
                    },
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

                return Ok(subgraph_response);
            };
        };

        let mut retry_budget = ctx.engine.retry_budget_for_subgraph(self.subgraph_id);

        if self.operation.ty.is_mutation()
            && subgraph.retry_config().and_then(|config| config.retry_mutations) != Some(true)
        {
            retry_budget = None;
        }

        execute_subgraph_request(
            ctx,
            span.clone(),
            self.subgraph_id,
            retry_budget,
            || FetchRequest {
                url: subgraph.url(),
                headers: ctx.subgraph_headers_with_rules(subgraph.header_rules()),
                json_body: Bytes::from(json_body.into_bytes()),
                timeout: subgraph.timeout(),
            },
            GraphqlIngester {
                ctx,
                plan,
                cache_ttl_and_key,
                subgraph_response,
            },
        )
        .instrument(span)
        .await
    }
}

fn build_cache_key(json_body: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(json_body.as_bytes());
    hasher.finalize().to_string()
}

struct GraphqlIngester<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    plan: PlanWalker<'ctx, (), ()>,
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
                response.next_seed(self.plan).ok_or("No object to update")?,
                RootGraphqlErrors {
                    response,
                    response_keys: self.plan.response_keys(),
                },
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
