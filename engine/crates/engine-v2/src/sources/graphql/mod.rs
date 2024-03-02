use runtime::fetch::FetchRequest;
use schema::{
    sources::federation::{RootFieldResolverWalker, SubgraphHeaderValueRef, SubgraphWalker},
    SubgraphId,
};

use self::query::PreparedGraphqlOperation;
use self::variables::OutboundVariables;

use super::{ExecutionContext, ExecutionResult, Executor, ExecutorInput, Plan};
use crate::{
    plan::{PlanWalker, PlanningResult},
    response::{ResponseBoundaryItem, ResponsePart},
};

mod deserialize;
mod federation;
mod query;
mod subscription;
mod variables;

pub(crate) use federation::*;
#[cfg(feature = "tracing")]
use grafbase_tracing::span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt, GqlResponseAttributes};
pub(crate) use subscription::*;

pub(crate) struct GraphqlExecutionPlan {
    subgraph_id: SubgraphId,
    operation: PreparedGraphqlOperation,
}

impl GraphqlExecutionPlan {
    pub fn build(resolver: RootFieldResolverWalker<'_>, plan: PlanWalker<'_>) -> PlanningResult<Plan> {
        let subgraph = resolver.subgraph();
        let operation =
            query::PreparedGraphqlOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(Plan::GraphQL(Self {
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
        let variables = OutboundVariables::new(plan.variables().collect());
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
        #[cfg(feature = "tracing")]
        let subgraph_request_span = SubgraphRequestSpan::new(self.subgraph.name())
            // TODO: atm this contains variables and we shouldn't be considering those. A follow up effort will take of this
            // .with_document(self.json_body.as_str())
            .into_span();

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
                                SubgraphHeaderValueRef::Forward(name) => {
                                    self.ctx.headers.get(name).and_then(|value| value.to_str().ok())?
                                }
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

        #[cfg(feature = "tracing")]
        subgraph_request_span.record_gql_response(GqlResponseAttributes {
            has_errors: self.response_part.has_errors(),
        });

        Ok(self.response_part)
    }
}
