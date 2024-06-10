use std::sync::Arc;

use grafbase_tracing::span::subgraph::SubgraphRequestSpan;
use runtime::fetch::FetchRequest;
use schema::{
    sources::graphql::{FederationEntityResolverWalker, GraphqlEndpointId, GraphqlEndpointWalker},
    HeaderValueRef,
};
use tracing::Instrument;

use crate::{
    execution::ExecutionContext,
    operation::OperationType,
    plan::{PlanWalker, PlanningResult},
    response::{ResponseBoundaryItem, ResponsePart},
    sources::{ExecutionResult, Executor, ExecutorInput, Plan},
};

use super::{
    deserialize::{ingest_deserializer_into_response, EntitiesDataSeed},
    query::PreparedFederationEntityOperation,
    variables::SubgraphVariables,
};

pub(crate) struct FederationEntityExecutionPlan {
    subgraph_id: GraphqlEndpointId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityExecutionPlan {
    pub fn build(resolver: FederationEntityResolverWalker<'_>, plan: PlanWalker<'_>) -> PlanningResult<Plan> {
        let subgraph = resolver.endpoint();
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(Plan::FederationEntity(Self {
            subgraph_id: subgraph.id(),
            operation,
        }))
    }

    pub fn new_executor<'ctx>(&'ctx self, input: ExecutorInput<'ctx, '_>) -> ExecutionResult<Executor<'ctx>> {
        let ExecutorInput {
            ctx,
            boundary_objects_view,
            plan,
            response_part,
        } = input;

        let boundary_objects_view = boundary_objects_view.with_extra_constant_fields(vec![(
            "__typename".to_string(),
            serde_json::Value::String(
                ctx.engine
                    .schema
                    .walker()
                    .walk(schema::Definition::from(plan.output().entity_type))
                    .name()
                    .to_string(),
            ),
        )]);
        let response_boundary_items = boundary_objects_view.items().clone();
        let variables = SubgraphVariables {
            plan,
            variables: &self.operation.variables,
            inputs: vec![(&self.operation.entities_variable_name, boundary_objects_view)],
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
            response_boundary_items,
            plan,
            response_part,
        }))
    }
}

pub(crate) struct FederationEntityExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedFederationEntityOperation,
    json_body: String,
    response_boundary_items: Arc<Vec<ResponseBoundaryItem>>,
    plan: PlanWalker<'ctx>,
    response_part: ResponsePart,
}

impl<'ctx> FederationEntityExecutor<'ctx> {
    #[tracing::instrument(skip_all, fields(plan_id = %self.plan.id(), federated_subgraph = %self.subgraph.name()))]
    pub async fn execute(mut self) -> ExecutionResult<ResponsePart> {
        let subgraph_request_span = SubgraphRequestSpan::new(self.subgraph.name())
            .with_url(self.subgraph.url())
            .with_operation_type(OperationType::Query.as_ref())
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
            let root_err_path = self
                .plan
                .root_error_path(&self.response_boundary_items[0].response_path);
            let seed_ctx = self.plan.new_seed(&mut self.response_part);
            ingest_deserializer_into_response(
                &seed_ctx,
                &root_err_path,
                EntitiesDataSeed {
                    ctx: seed_ctx.clone(),
                    response_boundary: &self.response_boundary_items,
                },
                &mut serde_json::Deserializer::from_slice(&bytes),
            );
            ExecutionResult::Ok(())
        }
        .instrument(subgraph_request_span.clone())
        .await?;

        Ok(self.response_part)
    }
}
