use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn simple() {
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

        let response = stream.call_tool("search", json!({"keywords": ["User"]})).await.unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "user",
                  "output_type": "User",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);
    });
}

#[test]
fn with_required_arguments() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                user(id: ID!): User
                searchUsers(query: String!, limit: Int = 10): [User!]!
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

        let response = stream.call_tool("search", json!({"keywords": ["user"]})).await.unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "user",
                  "output_type": "User",
                  "arguments": [
                    {
                      "name": "id",
                      "type": "ID!"
                    }
                  ]
                }
              ]
            },
            {
              "query_path": [
                {
                  "field": "searchUsers",
                  "output_type": "[User!]!",
                  "arguments": [
                    {
                      "name": "query",
                      "type": "String!"
                    },
                    {
                      "name": "limit",
                      "type": "Int",
                      "default_value": 10
                    }
                  ]
                }
              ]
            }
          ]
        }
        "#);
    });
}

#[test]
fn with_nested_types() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                post(id: ID!): Post
            }

            type Post {
                id: ID!
                title: String!
                author: User!
                comments(first: Int = 10, after: String): [Comment!]
                tags: [String!]!
            }

            type User {
                id: ID!
                name: String!
                email: String
            }

            type Comment {
                id: ID!
                body: String!
                author: User!
                createdAt: String!
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

        let response = stream.call_tool("search", json!({"keywords": ["post"]})).await.unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "post",
                  "output_type": "Post",
                  "arguments": [
                    {
                      "name": "id",
                      "type": "ID!"
                    }
                  ]
                }
              ]
            }
          ]
        }
        "#);

        // Search for nested fields
        let response = stream
            .call_tool("search", json!({"keywords": ["comments"]}))
            .await
            .unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "post",
                  "output_type": "Post",
                  "arguments": [
                    {
                      "name": "id",
                      "type": "ID!"
                    }
                  ]
                },
                {
                  "field": "comments",
                  "output_type": "[Comment!]",
                  "arguments": [
                    {
                      "name": "first",
                      "type": "Int",
                      "default_value": 10
                    },
                    {
                      "name": "after",
                      "type": "String"
                    }
                  ]
                }
              ]
            }
          ]
        }
        "#);
    });
}

#[test]
fn with_input_types() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                searchPosts(filter: PostFilter!, pagination: PaginationInput): [Post!]!
            }

            input PostFilter {
                title: String
                authorId: ID
                tags: [String!]
                createdAfter: String
            }

            input PaginationInput {
                first: Int! = 10
                after: String
            }

            type Post {
                id: ID!
                title: String!
                author: User!
                tags: [String!]!
                createdAt: String!
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
            .call_tool("search", json!({"keywords": ["searchPosts"]}))
            .await
            .unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "searchPosts",
                  "output_type": "[Post!]!",
                  "arguments": [
                    {
                      "name": "filter",
                      "type": "PostFilter!"
                    },
                    {
                      "name": "pagination",
                      "type": "PaginationInput"
                    }
                  ]
                }
              ]
            }
          ]
        }
        "#);
    });
}

#[test]
fn fuzzy_search() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                # 4-7 character words (1 typo allowed)
                users: [User!]!
                posts: [Post!]!
                # 8+ character words (2 typos allowed)
                comments: [Comment!]!
                articles: [Article!]!
                # Words that shouldn't match
                cat: String
                dog: String
            }

            type User {
                id: ID!
                name: String!
            }

            type Post {
                id: ID!
                title: String!
            }

            type Comment {
                id: ID!
                content: String!
            }

            type Article {
                id: ID!
                title: String!
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

        // Test 4-7 character words with 1 typo
        let response = stream.call_tool("search", json!({"keywords": ["user"]})).await.unwrap();
        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "users",
                  "output_type": "[User!]!",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);

        // Test 4-7 character words with 1 typo
        let response = stream.call_tool("search", json!({"keywords": ["post"]})).await.unwrap();
        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "posts",
                  "output_type": "[Post!]!",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);

        // Test 8+ character words with 2 typos
        let response = stream
            .call_tool("search", json!({"keywords": ["coment"]}))
            .await
            .unwrap();
        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "comments",
                  "output_type": "[Comment!]!",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);

        // Test 8+ character words with 2 typos
        let response = stream
            .call_tool("search", json!({"keywords": ["artcle"]}))
            .await
            .unwrap();
        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "articles",
                  "output_type": "[Article!]!",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);

        // Test words that shouldn't match (too short or too many typos)
        let response = stream
            .call_tool("search", json!({"keywords": ["ct", "dgo"]}))
            .await
            .unwrap();
        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": []
        }
        "#);
    });
}

#[test]
fn case_insensitive_search() {
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

        // Test case insensitive search
        let response = stream.call_tool("search", json!({"keywords": ["USER"]})).await.unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "user",
                  "output_type": "User",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);
    });
}

#[test]
fn multiple_keywords() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                user: User
                post: Post
                comment: Comment
            }

            type User {
                id: ID!
                name: String!
            }

            type Post {
                id: ID!
                title: String!
            }

            type Comment {
                id: ID!
                content: String!
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

        // Test search with multiple keywords
        let response = stream
            .call_tool("search", json!({"keywords": ["user", "post"]}))
            .await
            .unwrap();

        insta::assert_json_snapshot!(&response, @r#"
        {
          "fields": [
            {
              "query_path": [
                {
                  "field": "post",
                  "output_type": "Post",
                  "arguments": []
                }
              ]
            },
            {
              "query_path": [
                {
                  "field": "user",
                  "output_type": "User",
                  "arguments": []
                }
              ]
            }
          ]
        }
        "#);
    });
}
