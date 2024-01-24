use runtime::fetch::FetchRequest;
use schema::sources::federation::{RootFieldResolverWalker, SubgraphHeaderValueRef, SubgraphWalker};

use super::{ExecutionContext, Executor, ExecutorError, ExecutorResult, ResolverInput};
use crate::{
    plan::PlanOutput,
    response::{ExecutorOutput, ResponseBoundaryItem},
};

mod deserialize;
pub mod federation;
mod query;
mod subscription;

pub use subscription::*;

pub(crate) struct GraphqlExecutor<'ctx> {
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
        let err_path = self.boundary_item.response_path.child(
            self.ctx
                .walker
                .walk(self.plan_output.root_fields[0])
                .bound_response_key(),
        );

        let seed_ctx = self.ctx.seed_ctx(&mut self.output, &self.plan_output);
        deserialize::deserialize_response_into_output(
            &seed_ctx,
            &err_path,
            seed_ctx.create_root_seed(&self.boundary_item),
            &mut serde_json::Deserializer::from_slice(&bytes),
        );

        Ok(self.output)
    }
}
