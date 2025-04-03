use std::future::Future;

use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::Gateway, runtime};
use serde_json::json;

const SCHEMA: &str = r#"
type Query {
  node: Node
  nodes: [Node]!
  requiredNode: Node!
  listOfRequiredNodes: [Node!]!
}

interface Node {
    id: ID!
}

type A implements Node {
    id: ID!
}

type B implements Node @inaccessible {
    id: ID!
}
"#;

fn with_gateway<F: Future>(nodes: serde_json::Value, f: impl FnOnce(Gateway) -> F) -> F::Output {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(SCHEMA)
                    .with_resolver("Query", "node", nodes[0].clone())
                    .with_resolver("Query", "requiredNode", nodes[0].clone())
                    .with_resolver("Query", "nodes", nodes.clone())
                    .with_resolver("Query", "listOfRequiredNodes", nodes.clone())
                    .into_subgraph("test"),
            )
            .build()
            .await;
        f(gateway).await
    })
}

#[test]
fn accessible() {
    with_gateway(json!([{"__typename": "A", "id": "a"}]), |gateway| async move {
        let response = gateway
            .post(
                r#"{
                    node { id }
                    nodes { id }
                    requiredNode { id }
                    listOfRequiredNodes { id }
                }"#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "id": "a"
            },
            "nodes": [
              {
                "id": "a"
              }
            ],
            "requiredNode": {
              "id": "a"
            },
            "listOfRequiredNodes": [
              {
                "id": "a"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn inaccessible() {
    with_gateway(json!([{"__typename": "B", "id": "b"}]), |gateway| async move {
        let response = gateway
            .post(
                r#"{
                    node { id }
                }"#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          }
        }
        "#);

        let response = gateway
            .post(
                r#"{
                    nodes { id }
                }"#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nodes": [
              null
            ]
          }
        }
        "#);

        let response = gateway
            .post(
                r#"{
                    requiredNode { id }
                }"#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null
        }
        "#);

        let response = gateway
            .post(
                r#"{
                    listOfRequiredNodes { id }
                }"#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null
        }
        "#);
    });
}

#[test]
fn partially_inaccessible() {
    with_gateway(
        json!([{"__typename": "B", "id": "b"}, {"__typename": "A", "id": "a"}]),
        |gateway| async move {
            let response = gateway
                .post(
                    r#"{
                    node { id }
                }"#,
                )
                .await;
            insta::assert_json_snapshot!(response, @r#"
            {
              "data": {
                "node": null
              }
            }
            "#);

            let response = gateway
                .post(
                    r#"{
                    nodes { id }
                }"#,
                )
                .await;
            insta::assert_json_snapshot!(response, @r#"
            {
              "data": {
                "nodes": [
                  null,
                  {
                    "id": "a"
                  }
                ]
              }
            }
            "#);

            let response = gateway
                .post(
                    r#"{
                    requiredNode { id }
                }"#,
                )
                .await;
            insta::assert_json_snapshot!(response, @r#"
            {
              "data": null
            }
            "#);

            let response = gateway
                .post(
                    r#"{
                    listOfRequiredNodes { id }
                }"#,
                )
                .await;
            insta::assert_json_snapshot!(response, @r#"
            {
              "data": null
            }
            "#);
        },
    );
}

#[test]
fn inaccessible_extra() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(SCHEMA)
                    .with_resolver("Query", "node", json!({"__typename": "B", "id": "b"}))
                    .into_subgraph("x"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    node: Node @external
                    other: String @requires(fields: "node { id }")
                }

                interface Node @external {
                    id: ID!
                }
                "#,
                )
                .with_resolver("Query", "other", json!("yes"))
                .into_subgraph("y"),
            )
            .build()
            .await;

        let response = gateway.post("{ other }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "other": "yes"
          }
        }
        "#);
    })
}
