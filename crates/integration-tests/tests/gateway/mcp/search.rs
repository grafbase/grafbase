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

        let response = stream.call_tool("search", json!({"keywords": ["User"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.616214,
                  "field": {
                    "name": "user",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
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

        let response = stream.call_tool("search", json!({"keywords": ["user"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.1535835,
                  "field": {
                    "name": "user",
                    "type": "User",
                    "args": [
                      {
                        "name": "id",
                        "type": "ID!"
                      }
                    ]
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                },
                {
                  "score": 2.8213787,
                  "field": {
                    "name": "searchUsers",
                    "type": "[User!]!",
                    "args": [
                      {
                        "name": "query",
                        "type": "String!"
                      },
                      {
                        "name": "limit",
                        "type": "Int",
                        "defaultValue": 10
                      }
                    ]
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
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

        let response = stream.call_tool("search", json!({"keywords": ["post"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 7.052721,
                  "field": {
                    "name": "post",
                    "type": "Post",
                    "args": [
                      {
                        "name": "id",
                        "type": "ID!"
                      }
                    ]
                  },
                  "type": {
                    "name": "Post",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "author",
                        "type": "User!"
                      },
                      {
                        "name": "comments",
                        "type": "[Comment!]",
                        "args": [
                          {
                            "name": "first",
                            "type": "Int",
                            "defaultValue": 10
                          },
                          {
                            "name": "after",
                            "type": "String"
                          }
                        ]
                      },
                      {
                        "name": "id",
                        "type": "ID!"
                      },
                      {
                        "name": "tags",
                        "type": "[String!]!"
                      },
                      {
                        "name": "title",
                        "type": "String!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);

        // Search for nested fields
        let response = stream.call_tool("search", json!({"keywords": ["comments"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 3.6447704,
                  "field": {
                    "name": "comments",
                    "type": "[Comment!]",
                    "args": [
                      {
                        "name": "first",
                        "type": "Int",
                        "defaultValue": 10
                      },
                      {
                        "name": "after",
                        "type": "String"
                      }
                    ]
                  },
                  "type": {
                    "name": "Comment",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "author",
                        "type": "User!"
                      },
                      {
                        "name": "body",
                        "type": "String!"
                      },
                      {
                        "name": "createdAt",
                        "type": "String!"
                      },
                      {
                        "name": "id",
                        "type": "ID!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": [
                    {
                      "name": "post",
                      "type": "Post",
                      "args": [
                        {
                          "name": "id",
                          "type": "ID!"
                        }
                      ]
                    }
                  ]
                }
              ]
            ],
            "is_error": false
          }
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

        let response = stream.call_tool("search", json!({"keywords": ["searchPosts"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.134764,
                  "field": {
                    "name": "searchPosts",
                    "type": "[Post!]!",
                    "args": [
                      {
                        "name": "filter",
                        "type": "PostFilter!"
                      },
                      {
                        "name": "pagination",
                        "type": "PaginationInput"
                      }
                    ]
                  },
                  "type": {
                    "name": "Post",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "author",
                        "type": "User!"
                      },
                      {
                        "name": "createdAt",
                        "type": "String!"
                      },
                      {
                        "name": "id",
                        "type": "ID!"
                      },
                      {
                        "name": "tags",
                        "type": "[String!]!"
                      },
                      {
                        "name": "title",
                        "type": "String!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
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
        let response = stream.call_tool("search", json!({"keywords": ["user"]})).await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 3.5457788,
                  "field": {
                    "name": "users",
                    "type": "[User!]!"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);

        // Test 4-7 character words with 1 typo
        let response = stream.call_tool("search", json!({"keywords": ["post"]})).await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 3.5457788,
                  "field": {
                    "name": "posts",
                    "type": "[Post!]!"
                  },
                  "type": {
                    "name": "Post",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "id",
                        "type": "ID!"
                      },
                      {
                        "name": "title",
                        "type": "String!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);

        // Test 8+ character words with 2 typos
        let response = stream.call_tool("search", json!({"keywords": ["coment"]})).await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 1.0,
                  "field": {
                    "name": "comments",
                    "type": "[Comment!]!"
                  },
                  "type": {
                    "name": "Comment",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "content",
                        "type": "String!"
                      },
                      {
                        "name": "id",
                        "type": "ID!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);

        // Test 8+ character words with 2 typos
        let response = stream.call_tool("search", json!({"keywords": ["artcle"]})).await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 1.0,
                  "field": {
                    "name": "articles",
                    "type": "[Article!]!"
                  },
                  "type": {
                    "name": "Article",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "id",
                        "type": "ID!"
                      },
                      {
                        "name": "title",
                        "type": "String!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);

        // Test words that shouldn't match (too short or too many typos)
        let response = stream.call_tool("search", json!({"keywords": ["ct", "dgo"]})).await;
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
        let response = stream.call_tool("search", json!({"keywords": ["USER"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.616214,
                  "field": {
                    "name": "user",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
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
        let response = stream.call_tool("search", json!({"keywords": ["user", "post"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.889365,
                  "field": {
                    "name": "post",
                    "type": "Post"
                  },
                  "type": {
                    "name": "Post",
                    "kind": "OBJECT",
                    "fields": [
                      {
                        "name": "id",
                        "type": "ID!"
                      },
                      {
                        "name": "title",
                        "type": "String!"
                      }
                    ]
                  },
                  "root_type": "Query",
                  "ancestors": []
                },
                {
                  "score": 6.889365,
                  "field": {
                    "name": "user",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);
    });
}

#[test]
fn shallow_depth_should_be_first() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query {
                    posts: [Post]
                }

                type User {
                    id: ID!
                    name: String!
                }

                type Post {
                    id: ID!
                    title: String!
                    author: User
                    comments: [Comment]
                }

                type Comment {
                    id: ID!
                    content: String!
                    author: User
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
        let response = stream.call_tool("search", json!({"keywords": ["author"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 2.7161825,
                  "field": {
                    "name": "author",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": [
                    {
                      "name": "posts",
                      "type": "[Post]"
                    }
                  ]
                },
                {
                  "score": 1.8107883,
                  "field": {
                    "name": "author",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": [
                    {
                      "name": "posts",
                      "type": "[Post]"
                    },
                    {
                      "name": "comments",
                      "type": "[Comment]"
                    }
                  ]
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);
    });
}

#[test]
fn recursive_query() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                query: Query
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

        let response = stream.call_tool("search", json!({"keywords": ["User"]})).await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              [
                {
                  "score": 6.664409,
                  "field": {
                    "name": "user",
                    "type": "User"
                  },
                  "type": {
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
                  },
                  "root_type": "Query",
                  "ancestors": []
                }
              ]
            ],
            "is_error": false
          }
        }
        "#);
    });
}
