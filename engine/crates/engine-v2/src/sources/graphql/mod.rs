use grafbase_tracing::span::subgraph::SubgraphRequestSpan;
use runtime::fetch::FetchRequest;
use schema::{
    sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker, RootFieldResolverWalker},
    HeaderValueRef,
};
use serde::de::DeserializeSeed;
use tracing::Instrument;

use self::query::PreparedGraphqlOperation;
use self::variables::SubgraphVariables;

use super::{ExecutionContext, ExecutionResult, Executor, ExecutorInput, Plan};
use crate::{
    operation::OperationType,
    plan::{PlanWalker, PlanningResult},
    response::ResponsePart,
    sources::graphql::deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
};

mod deserialize;
mod federation;
mod query;
mod subscription;
mod variables;

pub(crate) use federation::*;
pub(crate) use subscription::*;

pub(crate) struct GraphqlExecutionPlan {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedGraphqlOperation,
}

impl GraphqlExecutionPlan {
    pub fn build(
        resolver: RootFieldResolverWalker<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Plan> {
        let subgraph = resolver.endpoint();
        let operation = query::PreparedGraphqlOperation::build(operation_type, plan)
            .map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(Plan::GraphQL(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    #[tracing::instrument(skip_all, fields(plan_id = %input.plan.id()))]
    pub fn new_executor<'ctx>(&'ctx self, input: ExecutorInput<'ctx, '_>) -> ExecutionResult<Executor<'ctx>> {
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

pub(crate) struct GraphqlExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedGraphqlOperation,
    json_body: String,
    plan: PlanWalker<'ctx>,
}

impl<'ctx> GraphqlExecutor<'ctx> {
    #[tracing::instrument(skip_all, fields(plan_id = %self.plan.id(), federated_subgraph = %self.subgraph.name()))]
    pub async fn execute(self, mut response_part: ResponsePart) -> ExecutionResult<ResponsePart> {
        let subgraph_request_span = SubgraphRequestSpan::new(self.subgraph.name())
            .with_operation_type(self.operation.ty.as_ref())
            // The query string contains no input values, only variables. So it's safe to log.
            .with_document(&self.operation.query)
            .into_span();

        async {
            let bytes = self
                .ctx
                .engine
                .env
                .fetcher
                .post(FetchRequest {
                    url: self.subgraph.url(),
                    json_body: self.json_body,
                    headers: self
                        .subgraph
                        .headers()
                        .filter_map(|header| {
                            Some((
                                header.name(),
                                match header.value() {
                                    HeaderValueRef::Forward(name) => self.ctx.header(name)?,
                                    HeaderValueRef::Static(value) => value,
                                },
                            ))
                        })
                        .collect(),
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
