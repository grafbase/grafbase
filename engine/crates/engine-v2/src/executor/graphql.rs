use schema::SubgraphResolver;

use super::{Executor, ExecutorRequest, ResponseProxy};
use crate::{response::ResponseObjectId, Engine};

pub(super) struct GraphqlExecutor {
    endpoint: String,
    query: String,
    variables: serde_json::Value,
    response_object_id: ResponseObjectId,
}

impl GraphqlExecutor {
    pub(super) fn build(engine: &Engine, resolver: &SubgraphResolver, request: ExecutorRequest<'_>) -> Executor {
        let SubgraphResolver { subgraph_id } = resolver;
        let subgraph = &engine.schema[*subgraph_id];
        Executor::GraphQL(GraphqlExecutor {
            endpoint: engine.schema[subgraph.url].clone(),
            query: String::new(),
            variables: serde_json::Value::Null,
            response_object_id: request.response_objects.id(),
        })
    }

    pub(super) async fn execute(self, proxy: ResponseProxy) {
        let response = reqwest::Client::new()
            .post(self.endpoint)
            .json(&serde_json::json!({
                "query": self.query,
                "variables": self.variables,
            }))
            .send()
            .await
            .unwrap();
        let bytes = response.bytes().await.unwrap();
        let object_node_id = self.response_object_id;
        proxy
            .mutate(move |response| {
                response
                    .insert_any(object_node_id, &mut serde_json::Deserializer::from_slice(&bytes))
                    .unwrap();
            })
            .await;
    }
}
