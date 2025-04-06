use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn test_object() {
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

        let response = stream.call_tool("introspect", json!({"types": ["User"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "User",
                  "kind": "OBJECT",
                  "fields": [
                    {
                      "name": "id",
                      "type": "ID!"
                    },
                    {
                      "name": "name",
                      "type": "String!"
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_union() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    search: SearchResult
                }

                union SearchResult = Post | Comment

                type Post {
                    id: ID!
                    title: String!
                }

                type Comment {
                    id: ID!
                    text: String!
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

        let response = stream.call_tool("introspect", json!({"types": ["SearchResult"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "SearchResult",
                  "kind": "UNION",
                  "possibleTypes": [
                    "Post",
                    "Comment"
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_interface() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    node: Node
                }

                interface Node {
                    id: ID!
                }

                type User implements Node {
                    id: ID!
                    name: String!
                }

                type Product implements Node {
                    id: ID!
                    price: Float!
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

        let response = stream.call_tool("introspect", json!({"types": ["Node"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "Node",
                  "kind": "INTERFACE",
                  "fields": [
                    {
                      "name": "id",
                      "type": "ID!"
                    }
                  ],
                  "possibleTypes": [
                    "User",
                    "Product"
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_enum() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    status: Status
                }

                enum Status {
                    DRAFT
                    PUBLISHED
                    ARCHIVED
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

        let response = stream.call_tool("introspect", json!({"types": ["Status"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "Status",
                  "kind": "ENUM",
                  "enumValues": [
                    {
                      "name": "DRAFT"
                    },
                    {
                      "name": "PUBLISHED"
                    },
                    {
                      "name": "ARCHIVED"
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_scalar() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    date: DateTime
                }

                scalar DateTime
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

        let response = stream.call_tool("introspect", json!({"types": ["DateTime"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "DateTime",
                  "kind": "SCALAR"
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_input_object() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    search(filter: SearchFilter): String
                }

                input SearchFilter {
                    term: String!
                    limit: Int
                    sortBy: String
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

        let response = stream.call_tool("introspect", json!({"types": ["SearchFilter"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "SearchFilter",
                  "kind": "INPUT_OBJECT",
                  "inputFields": [
                    {
                      "name": "term",
                      "type": "String!"
                    },
                    {
                      "name": "limit",
                      "type": "Int"
                    },
                    {
                      "name": "sortBy",
                      "type": "String"
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_object_with_field_arguments() {
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
                    posts(
                        first: Int = 10
                        offset: Int! 
                        status: PostStatus = PUBLISHED
                    ): [Post!]!
                }

                type Post {
                    id: ID!
                }

                enum PostStatus {
                    DRAFT
                    PUBLISHED
                    ARCHIVED
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

        let response = stream.call_tool("introspect", json!({"types": ["User"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "User",
                  "kind": "OBJECT",
                  "fields": [
                    {
                      "name": "id",
                      "type": "ID!"
                    },
                    {
                      "name": "posts",
                      "type": "[Post!]!",
                      "args": [
                        {
                          "name": "first",
                          "type": "Int",
                          "defaultValue": 10
                        },
                        {
                          "name": "offset",
                          "type": "Int!"
                        },
                        {
                          "name": "status",
                          "type": "PostStatus",
                          "defaultValue": "PUBLISHED"
                        }
                      ]
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn test_input_object_with_defaults() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    search(filter: SearchFilterWithDefaults): String
                }

                input SearchFilterWithDefaults {
                    term: String!
                    limit: Int = 50
                    sortBy: String = "createdAt"
                    includeArchived: Boolean = false
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
            .call_tool("introspect", json!({"types": ["SearchFilterWithDefaults"]}))
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "SearchFilterWithDefaults",
                  "kind": "INPUT_OBJECT",
                  "inputFields": [
                    {
                      "name": "term",
                      "type": "String!"
                    },
                    {
                      "name": "limit",
                      "type": "Int",
                      "defaultValue": 50
                    },
                    {
                      "name": "sortBy",
                      "type": "String",
                      "defaultValue": "createdAt"
                    },
                    {
                      "name": "includeArchived",
                      "type": "Boolean",
                      "defaultValue": false
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn should_not_show_mutations_if_disabled() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    user: User
                }

                type Mutation {
                    createUser: User
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
                enable_mutations = false
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream.call_tool("introspect", json!({"types": ["Mutation"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                "Type 'Mutation' not found"
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}

#[test]
fn should_show_mutations_if_enabled() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    user: User
                }

                type Mutation {
                    createUser: User
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
                enable_mutations = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream.call_tool("introspect", json!({"types": ["Mutation"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "name": "Mutation",
                  "kind": "OBJECT",
                  "fields": [
                    {
                      "name": "createUser",
                      "type": "User"
                    }
                  ]
                }
              ]
            ],
            "is_error": null
          }
        }
        "#);
    });
}
