use grafbase_tracing::span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt};
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
        let variables = SubgraphVariables {
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

        async {
            let bytes = ctx
                .engine
                .runtime
                .fetcher()
                .post(FetchRequest {
                    url: subgraph.url(),
                    json_body,
                    headers: ctx.headers_with_rules(subgraph.header_rules()),
                })
                .await?
                .bytes;

            tracing::debug!("{}", String::from_utf8_lossy(&bytes));

            let response = subgraph_response.as_mut();
            let status = GraphqlResponseSeed::new(
                response.next_seed(plan).ok_or("No object to update")?,
                RootGraphqlErrors {
                    response,
                    response_keys: plan.response_keys(),
                },
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

            span.record_gql_status(status);

            Ok(subgraph_response)
        }
        .instrument(span.clone())
        .await
    }
}
