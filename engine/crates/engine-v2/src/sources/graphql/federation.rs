use std::sync::Arc;

use runtime::fetch::FetchRequest;
use schema::{
    sources::federation::{EntityResolverWalker, SubgraphHeaderValueRef, SubgraphWalker},
    SubgraphId,
};

use crate::{
    execution::ExecutionContext,
    plan::{PlanWalker, PlanningResult},
    response::{ResponseBoundaryItem, ResponsePart},
    sources::{graphql::query::OutboundVariables, ExecutionPlan, ExecutionResult, Executor, ExecutorInput},
};

use super::{
    deserialize::{ingest_deserializer_into_response, EntitiesDataSeed},
    query::PreparedFederationEntityOperation,
};

pub(crate) struct FederationEntityExecutionPlan {
    subgraph_id: SubgraphId,
    operation: PreparedFederationEntityOperation,
}

impl FederationEntityExecutionPlan {
    pub fn build(resolver: EntityResolverWalker<'_>, plan: PlanWalker<'_>) -> PlanningResult<ExecutionPlan> {
        let subgraph = resolver.subgraph();
        let operation =
            PreparedFederationEntityOperation::build(plan).map_err(|err| format!("Failed to build query: {err}"))?;
        Ok(ExecutionPlan::FederationEntity(Self {
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
        let mut variables = OutboundVariables::new(&self.operation.variable_references, ctx.variables);
        variables
            .inputs
            .push((&self.operation.entities_variable, boundary_objects_view));

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
            json_body,
            response_boundary_items,
            plan,
            response_part,
        }))
    }
}

pub(crate) struct FederationEntityExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    response_boundary_items: Arc<Vec<ResponseBoundaryItem>>,
    plan: PlanWalker<'ctx>,
    response_part: ResponsePart,
}

impl<'ctx> FederationEntityExecutor<'ctx> {
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

        Ok(self.response_part)
    }
}
