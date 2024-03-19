use futures_util::{stream::BoxStream, StreamExt};
use runtime::fetch::GraphqlRequest;
use schema::sources::federation::{SubgraphHeaderValueRef, SubgraphWalker};

use super::{
    deserialize::ingest_deserializer_into_response, query::PreparedGraphqlOperation, variables::SubgraphVariables,
    ExecutionContext, GraphqlExecutionPlan,
};
use crate::{
    execution::OperationRootPlanExecution,
    plan::PlanWalker,
    sources::{ExecutionError, ExecutionResult, SubscriptionExecutor, SubscriptionInput},
};

pub(crate) struct GraphqlSubscriptionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    operation: &'ctx PreparedGraphqlOperation,
    plan: PlanWalker<'ctx>,
}

impl GraphqlExecutionPlan {
    pub fn new_subscription_executor<'ctx>(
        &'ctx self,
        input: SubscriptionInput<'ctx>,
    ) -> ExecutionResult<SubscriptionExecutor<'ctx>> {
        let SubscriptionInput { ctx, plan } = input;
        let subgraph = plan.schema().walk(self.subgraph_id);
        Ok(SubscriptionExecutor::Graphql(GraphqlSubscriptionExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            plan,
        }))
    }
}

impl<'ctx> GraphqlSubscriptionExecutor<'ctx> {
    pub async fn execute(
        self,
        new_execution: impl Fn() -> OperationRootPlanExecution<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, OperationRootPlanExecution<'ctx>>> {
        let Self {
            ctx,
            subgraph,
            operation,
            plan,
        } = self;

        let url = {
            let mut url = subgraph.websocket_url().clone();
            // If the user doesn't provide an explicit websocket URL we use the normal URL,
            // so make sure to convert the scheme to something appropriate
            match url.scheme() {
                "http" => url.set_scheme("ws").expect("this to work"),
                "https" => url.set_scheme("wss").expect("this to work"),
                _ => {}
            }
            url
        };

        let stream = ctx
            .engine
            .env
            .fetcher
            .stream(GraphqlRequest {
                url: &url,
                query: &operation.query,
                variables: serde_json::to_value(&SubgraphVariables {
                    plan,
                    variables: &operation.variables,
                    inputs: Vec::new(),
                })
                .map_err(|error| ExecutionError::Internal(error.to_string()))?,
                headers: subgraph
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
            .await?;

        Ok(Box::pin(
            stream
                .take_while(|result| std::future::ready(result.is_ok()))
                .map(move |response| {
                    let mut execution = new_execution();
                    ingest_response(
                        &mut execution,
                        plan,
                        response.expect("errors to be filtered out by the above take_while"),
                    );
                    execution
                }),
        ))
    }
}

fn ingest_response(
    execution: &mut OperationRootPlanExecution<'_>,
    plan: PlanWalker<'_>,
    subgraph_response: serde_json::Value,
) {
    let boundary_item = execution.root_response_boundary_item();
    let response_part = execution.root_response_part();

    let err_path = plan.root_error_path(&boundary_item.response_path);
    let seed_ctx = plan.new_seed(response_part);
    ingest_deserializer_into_response(
        &seed_ctx,
        &err_path,
        seed_ctx.create_root_seed(&boundary_item),
        subgraph_response,
    );
}
