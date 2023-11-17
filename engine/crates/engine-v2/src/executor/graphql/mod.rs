use schema::SubgraphResolver;

use super::{Executor, ExecutorContext, ExecutorError, ExecutorInput, ExecutorOutput};
use crate::{request::OperationSelectionSet, response::ResponseObjectId};

mod query;

#[derive(Debug)]
pub(super) struct GraphqlExecutor {
    endpoint_name: String,
    url: String,
    query: String,
    variables: serde_json::Value,
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

impl GraphqlExecutor {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build(
        ctx: ExecutorContext<'_>,
        resolver: &SubgraphResolver,
        selection_set: &OperationSelectionSet,
        input: ExecutorInput<'_>,
    ) -> Result<Executor<'static>, ExecutorError> {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &ctx.engine.schema[*subgraph_id];
        let query = query::QueryBuilder::build(ctx.operation.ty, ctx.default_walker().walk(selection_set)).unwrap();
        Ok(Executor::GraphQL(Self {
            endpoint_name: ctx.engine.schema[subgraph.name].to_string(),
            url: ctx.engine.schema[subgraph.url].clone(),
            query,
            variables: serde_json::Value::Null,
            response_object_id: input.response_object_roots.id(),
        }))
    }

    pub(super) async fn execute(
        self,
        _ctx: ExecutorContext<'_>,
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
            output.errors.add_simple_error(error.message);
        }

        Ok(())
    }
}
