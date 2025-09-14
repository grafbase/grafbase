use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

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

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream.call_tool("execute", json!({"query": "}"})).await;

        insta::assert_snapshot!(&response, @r#"
        Errors:
        At 1:1 unexpected closing brace ('}') token (expected one of , "{"query, mutation, subscription, fragment)
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
                    pets: [Pet]
                }

                type Pet {
                    name: String
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

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({
                    "query": "query { user { id email } }"
                }),
            )
            .await;

        insta::assert_snapshot!(&response, @r"
        Errors:
        At 1:19 User does not have a field named 'email'.

        == GraphQL SDL ==
        type User {
          id: ID!
          name: String!
          pets: [Pet]
        }

        type Pet {
          name: String
        }
        ");
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

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({
                    "query": "query { user }" // Missing selection set
                }),
            )
            .await;

        insta::assert_snapshot!(&response, @r"
        Errors:
        At 1:9 Leaf field 'user' must be a scalar or an enum, but is a User.

        == GraphQL SDL ==
        type User {
          id: ID!
          name: String!
        }
        ");
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

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({
                    "query": "query GetUser($id: Int!) { user(id: $id) { name } }",
                    "variables": { "id": 123 }
                }),
            )
            .await;

        insta::assert_snapshot!(&response, @r"
        Errors:
        At 1:37 Variable $id doesn't have the right type. Declared as 'Int!' but used as 'ID!'
        ");
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

        let mut stream = engine.mcp_http("/mcp").await;

        let response = stream
            .call_tool(
                "execute",
                json!({
                    "query": "query GetUser($id: ID!) { user(id: $userId) { name } }",
                    "variables": { "id": "123" }
                }),
            )
            .await;

        insta::assert_snapshot!(&response, @r"
        Errors:
        At 1:36 Unknown variable $userId
        ");
    });
}
