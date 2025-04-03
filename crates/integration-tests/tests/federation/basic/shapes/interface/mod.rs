mod one_interface;
mod one_union;
mod only_objects;
mod two_interfaces;

use std::future::Future;

use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::Gateway, runtime};

fn with_gateway<F: Future>(schema: &str, nodes: serde_json::Value, f: impl FnOnce(Gateway) -> F) -> F::Output {
    runtime().block_on(async move {
        let gateway = build(schema, nodes).await;
        f(gateway).await
    })
}

async fn build(schema: &str, nodes: serde_json::Value) -> Gateway {
    Gateway::builder()
        .with_subgraph(
            DynamicSchema::builder(
                [
                    r#"
                    type Query {
                      nodes: [Node!]!
                    }

                    interface Node {
                        id: ID!
                    }
                    "#,
                    schema,
                ]
                .join("\n"),
            )
            .with_resolver("Query", "nodes", nodes)
            .into_subgraph("test"),
        )
        .build()
        .await
}
