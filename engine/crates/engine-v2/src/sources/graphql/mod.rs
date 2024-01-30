use runtime::fetch::FetchRequest;
use schema::{
    sources::federation::{RootFieldResolverWalker, SubgraphHeaderValueRef, SubgraphWalker},
    SubgraphId,
};

use self::query::PreparedGraphqlOperation;

use super::{ExecutionContext, ExecutionPlan, ExecutionResult, Executor, ExecutorInput};
use crate::{
    plan::{PlanWalker, PlanningResult},
    response::{ResponseBoundaryItem, ResponsePart},
    sources::graphql::query::OutboundVariables,
};

mod deserialize;
mod federation;
mod query;
mod subscription;

pub(crate) use federation::*;
pub(crate) use subscription::*;

pub(crate) struct GraphqlExecutionPlan {
    subgraph_id: SubgraphId,
    operation: PreparedGraphqlOperation,
}

impl GraphqlExecutionPlan {
    pub fn build(resolver: RootFieldResolverWalker<'_>, plan: PlanWalker<'_>) -> PlanningResult<ExecutionPlan> {
        let subgraph = resolver.subgraph();
        let operation =
            query::PreparedGraphqlOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(ExecutionPlan::GraphQL(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    #[tracing::instrument(skip_all, fields(plan_id = %input.plan.id()))]
    pub fn new_executor<'ctx>(&'ctx self, input: ExecutorInput<'ctx, '_>) -> ExecutionResult<Executor<'ctx>> {
        let ExecutorInput {
            ctx,
            boundary_objects_view,
            plan,
            response_part,
        } = input;

        let subgraph = plan.schema().walk(self.subgraph_id);
        let variables = OutboundVariables::new(&self.operation.variable_references, ctx.variables);
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
            json_body,
            response_boundary_item: boundary_objects_view.into_single_boundary_item(),
            plan,
            response_part,
        }))
    }
}

pub(crate) struct GraphqlExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    response_boundary_item: ResponseBoundaryItem,
    plan: PlanWalker<'ctx>,
    response_part: ResponsePart,
}

impl<'ctx> GraphqlExecutor<'ctx> {
    #[tracing::instrument(skip_all, fields(plan_id = %self.plan.id(), federated_subgraph = %self.subgraph.name()))]
    pub async fn execute(mut self) -> ExecutionResult<ResponsePart> {
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
                                SubgraphHeaderValueRef::Forward(name) => self.ctx.header(name)?,
                                SubgraphHeaderValueRef::Static(value) => value,
                            },
                        ))
                    })
                    .collect(),
            })
            .await?
            .bytes;
        tracing::debug!("{}", String::from_utf8_lossy(&bytes));
        let err_path = self.plan.root_error_path(&self.response_boundary_item.response_path);
        let seed_ctx = self.plan.new_seed(&mut self.response_part);
        deserialize::ingest_deserializer_into_response(
            &seed_ctx,
            &err_path,
            seed_ctx.create_root_seed(&self.response_boundary_item),
            &mut serde_json::Deserializer::from_slice(&bytes),
        );

        Ok(self.response_part)
    }
}
