use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn valid() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
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
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool("verify", json!({"query": "query { user { name } }"}))
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              []
            ],
            "is_error": false
          }
        }
        "#);
    });
}

#[test]
fn unparseable_query() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
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
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream.call_tool("verify", json!({"query": "}"})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "unexpected closing brace ('}') token (expected one of , \"{\"query, mutation, subscription, fragment)"
              ]
            ],
            "is_error": true
          }
        }
        "#);
    });
}

#[test]
fn unknown_field() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
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
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool(
                "verify",
                json!({
                    "query": "query { user { id email } }"
                }),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "User does not have a field named 'email'. It has the following fields: id, name"
              ]
            ],
            "is_error": true
          }
        }
        "#);
    });
}

#[test]
fn invalid_query_structure() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
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
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool(
                "verify",
                json!({
                    "query": "query { user }" // Missing selection set
                }),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "Leaf field 'user' must be a scalar or an enum, but is a User."
              ]
            ],
            "is_error": true
          }
        }
        "#);
    });
}

#[test]
fn incorrect_variable_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    user(id: ID!): User
                }

                type User {
                    id: ID!
                    name: String!
                }
            "#,
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool(
                "verify",
                json!({
                    "query": "query GetUser($id: Int!) { user(id: $id) { name } }",
                    "variables": { "id": 123 }
                }),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "Variable $id doesn't have the right type. Declared as 'Int!' but used as 'ID!'"
              ]
            ],
            "is_error": true
          }
        }
        "#);
    });
}

#[test]
fn unknown_variable() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    user(id: ID!): User
                }

                type User {
                    id: ID!
                    name: String!
                }
            "#,
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool(
                "verify",
                json!({
                    "query": "query GetUser($id: ID!) { user(id: $userId) { name } }",
                    "variables": { "id": "123" }
                }),
            )
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "Unknown variable $userId"
              ]
            ],
            "is_error": true
          }
        }
        "#);
    });
}
