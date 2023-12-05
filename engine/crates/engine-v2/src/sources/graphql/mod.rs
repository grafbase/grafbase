use runtime::fetch::FetchRequest;
use schema::sources::federation::{RootFieldResolverWalker, SubgraphWalker};
use serde::de::DeserializeSeed;

use super::{ExecutionContext, Executor, ExecutorError, ExecutorResult, ResolverInput};
use crate::{
    plan::PlanOutput,
    response::{ExecutorOutput, ResponseBoundaryItem},
};

mod deserialize;
pub mod federation;
mod query;

pub struct GraphqlExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    boundary_item: ResponseBoundaryItem,
    plan_output: PlanOutput,
    output: ExecutorOutput,
}

impl<'ctx> GraphqlExecutor<'ctx> {
    pub fn build<'input>(
        resolver: RootFieldResolverWalker<'ctx>,
        ResolverInput {
            ctx,
            boundary_objects_view: roots,
            plan_id,
            plan_output,
            output,
        }: ResolverInput<'ctx, 'input>,
    ) -> ExecutorResult<Executor<'ctx>> {
        let subgraph = resolver.subgraph();
        let query = query::Query::build(ctx, plan_id, &plan_output)
            .map_err(|err| ExecutorError::Internal(format!("Failed to build query: {err}")))?;
        Ok(Executor::GraphQL(Self {
            ctx,
            subgraph,
            json_body: serde_json::to_string(&query)
                .map_err(|err| ExecutorError::Internal(format!("Failed to serialize query: {err}")))?,
            boundary_item: roots.into_single_boundary_item(),
            plan_output,
            output,
        }))
    }

    pub async fn execute(mut self) -> ExecutorResult<ExecutorOutput> {
        let bytes = self
            .ctx
            .engine
            .runtime
            .fetcher
            .post(FetchRequest {
                url: self.subgraph.url(),
                json_body: self.json_body,
            })
            .await?
            .bytes;
        let errors = deserialize::GraphqlResponseSeed::new(
            self.ctx
                .writer(&mut self.output, &self.boundary_item, &self.plan_output),
            Some(
                self.boundary_item
                    .response_path
                    .child(self.ctx.walker.walk(self.plan_output.fields[0]).bound_response_key),
            ),
        )
        .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))
        .expect("All errors have been handled.");

        self.output.push_errors(errors);

        Ok(self.output)
    }
}
