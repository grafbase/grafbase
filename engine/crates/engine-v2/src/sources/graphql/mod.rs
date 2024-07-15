use grafbase_telemetry::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker, RootFieldResolverWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;
use web_time::{Duration, Instant};

use self::query::PreparedGraphqlOperation;
use self::variables::SubgraphVariables;

use super::{ExecutionContext, ExecutionResult, Executor, ExecutorInput, PreparedExecutor};
use crate::{
    execution::{PlanWalker, PlanningResult},
    operation::OperationType,
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
        let span = SubgraphRequestSpan {
            name: self.subgraph.name(),
            operation_type: self.operation.ty.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: self.subgraph.url(),
        }
        .into_span();

        self.subgraph_request(&mut response_part)
            .instrument(span.clone())
            .await?;

        Ok(response_part)
    }

    async fn subgraph_request(self, response_part: &mut ResponsePart) -> ExecutionResult<()> {
        self.ctx
            .engine
            .runtime
            .rate_limiter()
            .limit(&crate::engine::RateLimitContext::Subgraph(self.subgraph.name()))
            .await?;

        let start = Instant::now();

        let response = self
            .ctx
            .engine
            .runtime
            .fetcher()
            .post(FetchRequest {
                url: self.subgraph.url(),
                headers: self.ctx.headers_with_rules(self.subgraph.header_rules()),
                json_body: self.json_body,
            })
            .await;

        let elapsed = start.elapsed();

        let response = match response {
            Ok(response) => response,
            Err(e) => {
                let status = SubgraphResponseStatus::HttpError;

                tracing::Span::current().record_subgraph_status(status, elapsed, Some(e.to_string()));
                tracing::error!(target: GRAFBASE_TARGET, "{e}");

                return Err(e.into());
            }
        };

        tracing::trace!("{}", String::from_utf8_lossy(&response.bytes));

        let part = response_part.as_mut();

        let result = GraphqlResponseSeed::new(
            part.next_seed(self.plan).ok_or("No object to update")?,
            RootGraphqlErrors {
                response_part: &part,
                response_keys: self.plan.response_keys(),
            },
        )
        .deserialize(&mut serde_json::Deserializer::from_slice(&response.bytes));

        handle_subgraph_result(result, response_part, elapsed)
    }
}

fn handle_subgraph_result(
    result: Result<GraphqlResponseStatus, serde_json::Error>,
    response_part: &ResponsePart,
    elapsed: Duration,
) -> ExecutionResult<()> {
    match result {
        result if response_part.blocked_in_planning() => {
            result?;
            Ok(())
        }
        Ok(status) if status.is_success() => {
            let subgraph_status = SubgraphResponseStatus::GraphqlResponse(status);

            tracing::Span::current().record_subgraph_status(subgraph_status, elapsed, None);
            tracing::debug!(target: GRAFBASE_TARGET, "subgraph response");

            Ok(())
        }
        Ok(status) => {
            let subgraph_status = SubgraphResponseStatus::GraphqlResponse(status);
            let message = response_part
                .first_error_message()
                .unwrap_or_else(|| String::from("subgraph error"));

            tracing::Span::current().record_subgraph_status(subgraph_status, elapsed, Some(message.clone()));
            tracing::error!(target: GRAFBASE_TARGET, "{message}");

            Ok(())
        }
        Err(e) => {
            let status = SubgraphResponseStatus::InvalidResponseError;
            let message = e.to_string();

            tracing::Span::current().record_subgraph_status(status, elapsed, Some(message.clone()));
            tracing::error!(target: GRAFBASE_TARGET, "{message}");

            Err(e.into())
        }
    }
}
