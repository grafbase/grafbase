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

        insta::assert_snapshot!(&response, @"");
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

        insta::assert_snapshot!(&response, @r#"
        {
          "errors": [
            "unexpected closing brace ('}') token (expected one of , \"{\"query, mutation, subscription, fragment)"
          ]
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

        insta::assert_snapshot!(&response, @r#"
        type User {
          id: ID!
          name: String!
        }


        ================================================================================

        {
          "errors": [
            "User does not have a field named 'email'."
          ]
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

        insta::assert_snapshot!(&response, @r#"
        type User {
          id: ID!
          name: String!
        }


        ================================================================================

        {
          "errors": [
            "Leaf field 'user' must be a scalar or an enum, but is a User."
          ]
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

        insta::assert_snapshot!(&response, @r#"
        {
          "errors": [
            "Variable $id doesn't have the right type. Declared as 'Int!' but used as 'ID!'"
          ]
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

        insta::assert_snapshot!(&response, @r#"
        {
          "errors": [
            "Unknown variable $userId"
          ]
        }
        "#);
    });
}
