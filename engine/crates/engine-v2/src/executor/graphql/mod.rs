use std::sync::Arc;

use futures_locks::Mutex;
use schema::SubgraphResolver;

use super::{Executor, ExecutorContext, ExecutorError};
use crate::{
    response::{Response, ResponseObjectId},
    Engine,
};

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
        engine: &Engine,
        resolver: &SubgraphResolver,
        ctx: ExecutorContext<'_>,
    ) -> Result<Executor, ExecutorError> {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &engine.schema[*subgraph_id];
        let query_builder = query::QueryBuilder::new(&engine.schema, &engine.schema, ctx.operation_fields);
        let query = query_builder.build(ctx.operation_type, ctx.selection_set).unwrap();
        Ok(Executor::GraphQL(Self {
            endpoint_name: engine.schema[subgraph.name].to_string(),
            url: engine.schema[subgraph.url].clone(),
            query,
            variables: serde_json::Value::Null,
            response_object_id: ctx.response_object_roots.id(),
        }))
    }

    pub(super) async fn execute(self, response: Arc<Mutex<Response>>) -> Result<(), ExecutorError> {
        let resp = reqwest::Client::new()
            .post(self.url)
            .json(&serde_json::json!({
                "query": self.query,
                "variables": self.variables,
            }))
            .send()
            .await
            .map_err(|err| format!("Request to '{}' failed with: {err}", self.endpoint_name))?;
        let bytes = resp.bytes().await.map_err(|_err| "Failed to read response")?;
        let gql_response: GraphqlResponse = serde_json::from_slice(&bytes).unwrap();
        let object_node_id = self.response_object_id;

        let mut response = response.lock().await;
        response.write_fields_json(object_node_id, gql_response.data);
        for error in gql_response.errors {
            response.add_simple_error(error.message);
        }

        Ok(())
    }
}
