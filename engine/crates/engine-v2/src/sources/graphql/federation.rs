use grafbase_telemetry::{
    gql_response_status::SubgraphResponseStatus,
    span::{subgraph::SubgraphRequestSpan, GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use runtime::fetch::FetchRequest;
use schema::sources::graphql::{FederationEntityResolverWalker, GraphqlEndpointId, GraphqlEndpointWalker};
use serde::de::DeserializeSeed;
use tracing::Instrument;
use web_time::Instant;

use crate::{
    execution::{ExecutionContext, PlanWalker, PlanningResult},
    operation::OperationType,
    response::ResponsePart,
    sources::{
        graphql::deserialize::{EntitiesErrorsSeed, GraphqlResponseSeed},
        ExecutionResult, Executor, ExecutorInput, PreparedExecutor,
    },
    Runtime,
};

use super::{deserialize::EntitiesDataSeed, query::PreparedFederationEntityOperation, variables::SubgraphVariables};

pub(crate) struct FederationEntityPreparedExecutor {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityPreparedExecutor {
    pub fn prepare(
        resolver: FederationEntityResolverWalker<'_>,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<PreparedExecutor> {
        let subgraph = resolver.endpoint();
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(PreparedExecutor::FederationEntity(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    pub fn new_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: ExecutorInput<'ctx, '_, R>,
    ) -> ExecutionResult<Executor<'ctx, R>> {
        let ExecutorInput {
            ctx,
            plan,
            root_response_objects,
        } = input;

        let root_response_objects = root_response_objects.with_extra_constant_fields(vec![(
            "__typename".to_string(),
            serde_json::Value::String(
                ctx.engine
                    .schema
                    .walker()
                    .walk(schema::Definition::from(plan.logical_plan().as_ref().entity_id))
                    .name()
                    .to_string(),
            ),
        )]);
        let variables = SubgraphVariables {
            plan,
            variables: &self.operation.variables,
            inputs: vec![(&self.operation.entities_variable_name, root_response_objects)],
        };

        let subgraph = ctx.engine.schema.walk(self.subgraph_id);
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

        Ok(Executor::FederationEntity(FederationEntityExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            json_body,
            plan,
        }))
    }
}

pub(crate) struct FederationEntityExecutor<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedFederationEntityOperation,
    json_body: String,
    plan: PlanWalker<'ctx>,
}

impl<'ctx, R: Runtime> FederationEntityExecutor<'ctx, R> {
    #[tracing::instrument(skip_all)]
    pub async fn execute(self, mut response_part: ResponsePart) -> ExecutionResult<ResponsePart> {
        let span = SubgraphRequestSpan {
            name: self.subgraph.name(),
            operation_type: OperationType::Query.as_str(),
            // The generated query does not contain any data, everything are in the variables, so
            // it's safe to use.
            sanitized_query: &self.operation.query,
            url: self.subgraph.url(),
        }
        .into_span();

        self.subgraph_request(&mut response_part).instrument(span).await?;

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
                tracing::error!(GRAFBASE_TARGET, "{e}");

                return Err(e.into());
            }
        };

        tracing::trace!("{}", String::from_utf8_lossy(&response.bytes));

        let part = response_part.as_mut();

        let result = GraphqlResponseSeed::new(
            EntitiesDataSeed {
                response_part: &part,
                plan: self.plan,
            },
            EntitiesErrorsSeed {
                response_part: &part,
                response_keys: self.plan.response_keys(),
            },
        )
        .deserialize(&mut serde_json::Deserializer::from_slice(&response.bytes));

        super::handle_subgraph_result(result, response_part, elapsed)
    }
}
