use std::collections::HashMap;

use engine_value::ConstValue;
use schema::SubgraphResolver;

use self::query::Query;
use super::{ExecutionContext, Executor, ExecutorError, ExecutorInput, ExecutorOutput};
use crate::response::ResponseObjectId;

mod query;

#[derive(Debug)]
pub struct GraphqlExecutor<'a> {
    endpoint_name: String,
    url: String,
    query: String,
    variables: HashMap<String, &'a ConstValue>,
    response_object_id: ResponseObjectId,
}

#[derive(serde::Deserialize)]
struct GraphqlResponse {
    data: serde_json::Value,
    #[serde(default)]
    errors: Vec<GraphqlError>,
}

#[derive(serde::Deserialize)]
struct GraphqlError {
    message: String,
}

impl<'a> GraphqlExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx>,
        resolver: &SubgraphResolver,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &ctx.engine.schema[*subgraph_id];
        let Query { query, variables } = query::QueryBuilder::build(
            &ctx.plan.operation,
            ctx.plan_id,
            ctx.default_walk_variables(),
            ctx.default_walk_selection_set(),
        )
        .map_err(|err| ExecutorError::InternalError(format!("Failed to build query: {err}")))?;
        Ok(Executor::GraphQL(Self {
            endpoint_name: ctx.engine.schema[subgraph.name].to_string(),
            url: ctx.engine.schema[subgraph.url].clone(),
            query,
            variables,
            response_object_id: input.root_response_objects.root().id,
        }))
    }

    pub(super) async fn execute(
        self,
        _ctx: ExecutionContext<'_>,
        output: &mut ExecutorOutput,
    ) -> Result<(), ExecutorError> {
        let response: GraphqlResponse = {
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
            serde_json::from_slice(&bytes).unwrap()
        };

        output
            .data
            .lock()
            .await
            .write_fields_json(self.response_object_id, response.data);
        for error in response.errors {
            output
                .errors
                .add_simple_error(format!("Upstream error: {}", error.message));
        }

        Ok(())
    }
}
