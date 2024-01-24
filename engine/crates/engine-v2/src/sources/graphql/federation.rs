use runtime::fetch::FetchRequest;
use schema::sources::federation::{EntityResolverWalker, SubgraphHeaderValueRef, SubgraphWalker};

use crate::{
    execution::ExecutionContext,
    plan::PlanOutput,
    request::EntityType,
    response::{ExecutorOutput, ResponseBoundaryItem},
    sources::{Executor, ExecutorError, ExecutorResult, ResolverInput},
};

use super::{
    deserialize::{deserialize_response_into_output, EntitiesDataSeed},
    query,
};

pub(crate) struct FederationEntityExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    response_boundary: Vec<ResponseBoundaryItem>,
    plan_output: PlanOutput,
    output: ExecutorOutput,
}

impl<'ctx> FederationEntityExecutor<'ctx> {
    pub fn build<'input>(
        resolver: EntityResolverWalker<'ctx>,
        entity_type: EntityType,
        ResolverInput {
            ctx,
            boundary_objects_view,
            plan_id,
            plan_output,
            output,
        }: ResolverInput<'ctx, 'input>,
    ) -> ExecutorResult<Executor<'ctx>> {
        let subgraph = resolver.subgraph();
        let boundary_objects_view = boundary_objects_view.with_extra_constant_fields(vec![(
            "__typename".to_string(),
            serde_json::Value::String(
                ctx.schema()
                    .walk(schema::Definition::from(entity_type))
                    .name()
                    .to_string(),
            ),
        )]);
        let response_boundary = boundary_objects_view.boundary();
        let query = query::FederationEntityQuery::build(ctx, plan_id, &plan_output, boundary_objects_view)
            .map_err(|err| ExecutorError::Internal(format!("Failed to build query: {err}")))?;
        Ok(Executor::FederationEntity(Self {
            ctx,
            subgraph,
            json_body: serde_json::to_string(&query)
                .map_err(|err| ExecutorError::Internal(format!("Failed to serialize query: {err}")))?,
            response_boundary,
            plan_output,
            output,
        }))
    }

    pub async fn execute(mut self) -> ExecutorResult<ExecutorOutput> {
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
        let err_path = self.response_boundary[0].response_path.child(
            self.ctx
                .walker
                .walk(self.plan_output.root_fields[0])
                .bound_response_key(),
        );
        let seed_ctx = self.ctx.seed_ctx(&mut self.output, &self.plan_output);
        deserialize_response_into_output(
            &seed_ctx,
            &err_path,
            EntitiesDataSeed {
                ctx: seed_ctx.clone(),
                response_boundary: &self.response_boundary,
            },
            &mut serde_json::Deserializer::from_slice(&bytes),
        );

        Ok(self.output)
    }
}
