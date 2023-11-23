use std::collections::HashMap;

use engine_value::ConstValue;
use schema::SubgraphResolver;
use serde::de::DeserializeSeed;

use super::{ExecutionContext, Executor, ExecutorError, ExecutorInput};
use crate::response::{ResponseObjectRoot, ResponsePartBuilder};

mod deserialize;
mod query;

#[derive(Debug)]
pub struct GraphqlExecutor<'a> {
    endpoint_name: String,
    url: String,
    query: String,
    variables: HashMap<String, &'a ConstValue>,
    response_object_root: ResponseObjectRoot,
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
                .map_err(|err| ExecutorError::InternalError(format!("Failed to build query: {err}")))?;
        Ok(Executor::GraphQL(Self {
            endpoint_name: ctx.engine.schema[subgraph.name].to_string(),
            url: ctx.engine.schema[subgraph.url].clone(),
            query,
            variables,
            response_object_root: input.root_response_objects.root(),
        }))
    }

    pub(super) async fn execute(
        self,
        ctx: ExecutionContext<'_, '_>,
        output: &mut ResponsePartBuilder,
    ) -> Result<(), ExecutorError> {
        let response = reqwest::Client::new()
            .post(self.url)
            .json(&serde_json::json!({
                "query": self.query,
                "variables": self.variables,
            }))
            .send()
            .await
            .map_err(|err| format!("Request to '{}' failed with: {err}", self.endpoint_name))?;
        let bytes = response.bytes().await.map_err(|_err| "Failed to read response")?;
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
