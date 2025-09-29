use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{Gateway, GatewayBuilder},
    runtime,
};
use serde_json::json;

fn builder() -> GatewayBuilder {
    Gateway::builder()
        .with_subgraph(
            DynamicSchema::builder(
                r#"
                extend schema @link(url: "authz-19-subgraph-grouping", import: ["@auth"])

                type Query {
                    publicA: String
                    privateA: String @auth
                    commonA: Common
                    anyA: Any
                }

                union Any @shareable = Common | Nested 

                type Common @shareable {
                    nested: Nested
                    value: String @auth
                }

                type Nested @shareable @auth {
                    nestedValue: String
                }
                "#,
            )
            .with_resolver("Query", "publicA", "publicA")
            .with_resolver("Query", "privateA", "privateA")
            .with_resolver(
                "Query",
                "commonA",
                json!({"value": "a", "nested": {"nestedValue": "nestedA"}}),
            )
            .with_resolver(
                "Query",
                "anyA",
                json!({"nestedValue": "nestedA", "__typename": "Nested"}),
            )
            .into_subgraph("A"),
        )
        .with_subgraph(
            DynamicSchema::builder(
                r#"
                extend schema @link(url: "authz-19-subgraph-grouping", import: ["@auth"])

                type Query {
                    publicB: String
                    privateB: String @auth
                    commonB: Common
                    anyB: Any
                }

                union Any @shareable = Common | Nested 

                type Common @shareable {
                    nested: Nested
                    value: String @auth
                }

                type Nested @shareable @auth {
                    nestedValue: String
                }
                "#,
            )
            .with_resolver("Query", "publicB", "publicB")
            .with_resolver("Query", "privateB", "privateB")
            .with_resolver(
                "Query",
                "commonB",
                json!({"value": "b", "nested": {"nestedValue": "nestedB"}}),
            )
            .with_resolver(
                "Query",
                "anyB",
                json!({"nestedValue": "nestedB", "__typename": "Nested"}),
            )
            .into_subgraph("B"),
        )
        .with_extension("authz-19-subgraph-grouping")
}

const QUERY: &str = "query { publicA privateA commonA { value } anyA { ... on Nested { nestedValue } } publicB privateB commonB { value } anyB { ... on Nested { nestedValue } } }";

#[test]
fn all_allowed() {
    runtime().block_on(async move {
        let engine = builder().build().await;
        let response = engine.post(QUERY).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "publicA": "publicA",
            "privateA": "privateA",
            "commonA": {
              "value": "a"
            },
            "anyA": {
              "nestedValue": "nestedA"
            },
            "publicB": "publicB",
            "privateB": "privateB",
            "commonB": {
              "value": "b"
            },
            "anyB": {
              "nestedValue": "nestedB"
            }
          }
        }
        "#);
        let sent = engine.drain_graphql_requests_sent_to_by_name("A");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { publicA privateA commonA { value } anyA { __typename ... on Nested { nestedValue } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);
        let sent = engine.drain_graphql_requests_sent_to_by_name("B");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { publicB privateB commonB { value } anyB { __typename ... on Nested { nestedValue } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);
    });
}

#[test]
fn one_denied() {
    runtime().block_on(async move {
        let engine = builder()
            .with_toml_config(
                r#"
                [extensions.authz-19-subgraph-grouping.config]
                denied_subgraph_names = ["A"]
                "#,
            )
            .build()
            .await;

        let response = engine.post(QUERY).await;
        // FIXME: anyA.nestedValue shouldn't be null, it's anyA that should be.
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "publicA": "publicA",
            "privateA": null,
            "commonA": {
              "value": null
            },
            "anyA": {
              "nestedValue": null
            },
            "publicB": "publicB",
            "privateB": "privateB",
            "commonB": {
              "value": "b"
            },
            "anyB": {
              "nestedValue": "nestedB"
            }
          },
          "errors": [
            {
              "message": "Not authorized, denied subgraph SDK19",
              "locations": [
                {
                  "line": 1,
                  "column": 36
                }
              ],
              "path": [
                "commonA",
                "value"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("A");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { publicA commonA { __typename @skip(if: true) } anyA { __typename } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("B");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { publicB privateB commonB { value } anyB { __typename ... on Nested { nestedValue } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#);
    });
}
