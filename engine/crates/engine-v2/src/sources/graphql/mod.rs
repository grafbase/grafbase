use grafbase_tracing::span::subgraph::SubgraphRequestSpan;
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker, RootFieldResolverWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use self::query::PreparedGraphqlOperation;
use self::variables::SubgraphVariables;

use super::{ExecutionContext, ExecutionResult, Executor, ExecutorInput, PreparedExecutor};
use crate::{
    operation::OperationType,
    plan::{PlanWalker, PlanningResult},
    response::ResponsePart,
    sources::graphql::deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    Runtime,
};

mod deserialize;
mod federation;
mod query;
mod subscription;
mod variables;

pub(crate) use federation::*;
pub(crate) use subscription::*;

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
    pub fn new_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: ExecutorInput<'ctx, '_, R>,
    ) -> ExecutionResult<Executor<'ctx, R>> {
        let ExecutorInput { ctx, plan, .. } = input;

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

        Ok(Executor::GraphQL(GraphqlExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            json_body,
            plan,
        }))
    }
}

pub(crate) struct GraphqlExecutor<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedGraphqlOperation,
    json_body: String,
    plan: PlanWalker<'ctx>,
}

impl<'ctx, R: Runtime> GraphqlExecutor<'ctx, R> {
    #[tracing::instrument(skip_all)]
    pub async fn execute(self, mut response_part: ResponsePart) -> ExecutionResult<ResponsePart> {
        let subgraph_request_span = SubgraphRequestSpan::new(self.subgraph.name())
            .with_url(self.subgraph.url())
            .with_operation_type(self.operation.ty.as_ref())
            // The query string contains no input values, only variables. So it's safe to log.
            .with_document(&self.operation.query)
            .into_span();

        async {
            let bytes = self
                .ctx
                .engine
                .runtime
                .fetcher()
                .post(FetchRequest {
                    url: self.subgraph.url(),
                    json_body: self.json_body,
                    headers: self.ctx.headers_with_rules(self.subgraph.headers()),
                })
                .await?
                .bytes;

            tracing::debug!("{}", String::from_utf8_lossy(&bytes));

            let part = response_part.as_mut();

            GraphqlResponseSeed::new(
                part.next_seed(self.plan).ok_or("No object to update")?,
                RootGraphqlErrors {
                    response_part: &part,
                    response_keys: self.plan.response_keys(),
                },
            )
            .with_graphql_span(subgraph_request_span.clone())
            .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))?;

            Ok(response_part)
        }
        .instrument(subgraph_request_span.clone())
        .await
    }
}
