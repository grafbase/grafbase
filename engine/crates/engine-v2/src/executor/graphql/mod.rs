use std::collections::HashMap;

use engine_value::ConstValue;
use runtime::fetch::FetchRequest;
use schema::SubgraphResolver;
use serde::de::DeserializeSeed;

use super::{ExecutionContext, Executor, ExecutorError, ExecutorInput};
use crate::response::{ResponseObjectRoot, ResponsePartBuilder};

mod deserialize;
mod query;

#[derive(Debug)]
pub struct GraphqlExecutor<'a> {
    url: String,
    payload: Payload<'a>,
    response_object_root: ResponseObjectRoot,
}

#[derive(Debug, serde::Serialize)]
pub struct Payload<'a> {
    query: String,
    variables: HashMap<String, &'a ConstValue>,
}

impl<'a> GraphqlExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx, 'ctx>,
        resolver: &SubgraphResolver,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &ctx.engine.schema[*subgraph_id];
        let query::Query { query, variables } =
            query::QueryBuilder::build(ctx.operation, ctx.plan_id, ctx.variables(), ctx.selection_set())
                .map_err(|err| ExecutorError::Internal(format!("Failed to build query: {err}")))?;
        Ok(Executor::GraphQL(Self {
            url: ctx.engine.schema[subgraph.url].clone(),
            payload: Payload { query, variables },
            response_object_root: input.root_response_objects.root(),
        }))
    }

    pub(super) async fn execute(
        self,
        ctx: ExecutionContext<'_, '_>,
        output: &mut ResponsePartBuilder,
    ) -> Result<(), ExecutorError> {
        let bytes = ctx
            .engine
            .runtime
            .fetcher
            .post(FetchRequest {
                url: &self.url,
                json_body: serde_json::to_string(&self.payload).unwrap(),
            })
            .await?
            .bytes;

        deserialize::UniqueRootSeed {
            ctx: &ctx,
            output,
            root: &self.response_object_root,
        }
        .deserialize(&mut serde_json::Deserializer::from_slice(&bytes))
        .map_err(|err| format!("Deserialization failure: {err}"))?;

        Ok(())
    }
}
