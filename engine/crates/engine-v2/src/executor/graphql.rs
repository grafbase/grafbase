use schema::SubgraphResolver;

use super::{Executor, ExecutorError, ExecutorRequest, ResponseProxy};
use crate::{response::ResponseObjectId, Engine};

#[derive(Debug)]
pub(super) struct GraphqlExecutor {
    url: String,
    query: String,
    variables: serde_json::Value,
    response_object_id: ResponseObjectId,
}

impl GraphqlExecutor {
    pub(super) fn build(engine: &Engine, resolver: &SubgraphResolver, request: ExecutorRequest<'_>) -> Executor {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &engine.schema[*subgraph_id];
        Executor::GraphQL(Self {
            url: engine.schema[subgraph.url].clone(),
            query: String::new(),
            variables: serde_json::Value::Null,
            response_object_id: request.response_objects.id(),
        })
    }

    pub(super) async fn execute(self, proxy: ResponseProxy) -> Result<(), ExecutorError> {
        let response = reqwest::Client::new()
            .post(self.url)
            .json(&serde_json::json!({
                "query": self.query,
                "variables": self.variables,
            }))
            .send()
            .await
            .map_err(|_err| "Request failed")?;
        let bytes = response.bytes().await.map_err(|_err| "Failed to read response")?;
        let object_node_id = self.response_object_id;
        proxy
            .mutate(move |response| {
                response.write_fields_any(object_node_id, &mut serde_json::Deserializer::from_slice(&bytes))
            })
            .await
            .map_err(|_err| "Failed to write response")?;
        Ok(())
    }
}
