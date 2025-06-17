use graphql_mocks::{EchoSchema, dynamic::DynamicSchema};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn execute_valid_graphql_query() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    user: User
                }

                type User {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .with_resolver("Query", "user", json!({"id": "1", "name": "Alice"}))
                .into_subgraph("y"),
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool("execute", json!({"query": "query { user { name } }"}))
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              {
                "data": {
                  "user": {
                    "name": "Alice"
                  }
                }
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn execute_mutation_is_rejected() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    user: User
                }

                type Mutation {
                    updateUser(name: String!): User
                }

                type User {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .into_subgraph("y"),
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({"query": "mutation { updateUser(name: \"Bob\") { name } }"}),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              {
                "errors": [
                  {
                    "message": "Mutation is not allowed with a safe method like GET",
                    "extensions": {
                      "code": "BAD_REQUEST"
                    }
                  }
                ]
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn execute_mutation_is_accepted_if_configured() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    user: User
                }

                type Mutation {
                    updateUser(name: String!): User
                }

                type User {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .with_resolver("Mutation", "updateUser", json!({"id": "1", "name": "Alice"}))
                .into_subgraph("y"),
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
                execute_mutations = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({"query": "mutation { updateUser(name: \"Bob\") { name } }"}),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              {
                "data": {
                  "updateUser": {
                    "name": "Alice"
                  }
                }
              }
            ]
          }
        }
        "#);
    });
}

#[tokio::test]
async fn execute_with_header() {
    let engine = Gateway::builder()
        .with_subgraph(EchoSchema)
        .with_toml_config(
            r#"
            [mcp]
            enabled = true

            [[headers]]
            rule = "forward"
            name = "authorization"

            [[headers]]
            rule = "forward"
            name = "x-test"
        "#,
        )
        .build()
        .await;

    let mut headers = http::HeaderMap::new();

    headers.insert("AUTHORIZATION", "Bearer token".parse().unwrap());
    headers.insert("x-test", "test".parse().unwrap());

    let mut stream = engine.mcp_http("/mcp").with_headers(headers).await;

    let response = stream
        .call_tool(
            "execute",
            json!({"query": "query {
                authorization: header(name: \"authorization\")
                xTest: header(name: \"x-test\")
            }"}),
        )
        .await;

    insta::assert_json_snapshot!(&response, @r#"
    {
      "result": {
        "content": [
          {
            "data": {
              "authorization": "Bearer token",
              "xTest": "test"
            }
          }
        ]
      }
    }
    "#);
}
