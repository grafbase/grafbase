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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          user: User
        }
        "##);
    });
}

#[test]
fn camel_case_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                anything: UserWithData
            }

            type UserWithData {
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
        insta::assert_snapshot!(&response, @r##"
        type UserWithData {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          anything: UserWithData
        }
        "##);
    });
}

#[test]
fn camel_case_field() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                userWithData: Anything
            }

            type Anything {
                id: ID!
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
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          userWithData: Anything
        }

        type Anything {
          id: ID!
        }
        "##);
    });
}

#[test]
fn acronym_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
            type Query {
                anything: HTTPRequest
            }

            type HTTPRequest {
                id: ID!
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

        let response = stream.call_tool("search", json!({"keywords": ["http"]})).await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          anything: HTTPRequest
        }

        type HTTPRequest {
          id: ID!
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          user(id: ID!): User
          searchUsers(query: String!, limit: Int = 10): [User!]!
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type Post {
          author: User!
          comments(first: Int = 10, after: String): [Comment!]
          id: ID!
          tags: [String!]!
          title: String!
        }

        # Incomplete fields
        type Query {
          post(id: ID!): Post
        }

        type User {
          email: String
          id: ID!
          name: String!
        }

        type Comment {
          author: User!
          body: String!
          createdAt: String!
          id: ID!
        }
        "##);

        // Search for nested fields
        let response = stream.call_tool("search", json!({"keywords": ["comments"]})).await;
        insta::assert_snapshot!(&response, @r##"
        type Comment {
          author: User!
          body: String!
          createdAt: String!
          id: ID!
        }

        # Incomplete fields
        type Query {
          post(id: ID!): Post
        }

        type Post {
          author: User!
          comments(first: Int = 10, after: String): [Comment!]
          id: ID!
          tags: [String!]!
          title: String!
        }

        type User {
          email: String
          id: ID!
          name: String!
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          searchPosts(filter: PostFilter!, pagination: PaginationInput): [Post!]!
        }

        type Post {
          author: User!
          createdAt: String!
          id: ID!
          tags: [String!]!
          title: String!
        }

        type User {
          id: ID!
          name: String!
        }

        input PaginationInput {
          first: Int! = 10
          after: String
        }

        input PostFilter {
          title: String
          authorId: ID
          tags: [String!]
          createdAfter: String
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          users: [User!]!
        }
        "##);

        // Test 4-7 character words with 1 typo
        let response = stream.call_tool("search", json!({"keywords": ["post"]})).await;
        insta::assert_snapshot!(&response, @r##"
        type Post {
          id: ID!
          title: String!
        }

        # Incomplete fields
        type Query {
          posts: [Post!]!
        }
        "##);

        // Test 8+ character words with 2 typos
        let response = stream.call_tool("search", json!({"keywords": ["coment"]})).await;
        insta::assert_snapshot!(&response, @r##"
        type Comment {
          content: String!
          id: ID!
        }

        # Incomplete fields
        type Query {
          comments: [Comment!]!
        }
        "##);

        // Test 8+ character words with 2 typos
        let response = stream.call_tool("search", json!({"keywords": ["artcle"]})).await;
        insta::assert_snapshot!(&response, @r##"
        type Article {
          id: ID!
          title: String!
        }

        # Incomplete fields
        type Query {
          articles: [Article!]!
        }
        "##);

        // Test words that shouldn't match (too short or too many typos)
        let response = stream.call_tool("search", json!({"keywords": ["ct", "dgo"]})).await;
        insta::assert_snapshot!(&response, @"");
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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          user: User
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          user: User
          post: Post
        }

        type Post {
          id: ID!
          title: String!
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type Comment {
          author: User
          content: String!
          id: ID!
        }

        type Post {
          author: User
          comments: [Comment]
          id: ID!
          title: String!
        }

        # Incomplete fields
        type Query {
          posts: [Post]
        }

        type User {
          id: ID!
          name: String!
        }
        "##);
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
        insta::assert_snapshot!(&response, @r##"
        type User {
          id: ID!
          name: String!
        }

        # Incomplete fields
        type Query {
          user: User
        }
        "##);
    });
}

#[test]
fn search_descriptions() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                """
                Root query type for the API
                """
                type Query {
                    """
                    Search for blog posts using various criteria
                    """
                    searchPosts(
                        """
                        Filter criteria for blog posts
                        """
                        filter: PostFilter!
                    ): [Post!]!
                }

                """
                Input type for filtering blog posts
                """
                input PostFilter {
                    """
                    Search by post title (case insensitive)
                    """
                    title: String

                    """
                    Filter posts by specific tags
                    """
                    tags: [String!]

                    """
                    Only return posts created after this date
                    """
                    createdAfter: String
                }

                """
                Represents a blog post in the system
                """
                type Post {
                    id: ID!
                    title: String!
                    """
                    The main content/body of the blog post
                    """
                    content: String!
                    """
                    List of tags associated with the post
                    """
                    tags: [String!]!
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

        // Search for a term that appears in descriptions
        let response = stream.call_tool("search", json!({"keywords": ["blog"]})).await;
        insta::assert_snapshot!(&response, @r##"
        "Represents a blog post in the system"
        type Post {
          "The main content/body of the blog post"
          content: String!
          createdAt: String!
          id: ID!
          "List of tags associated with the post"
          tags: [String!]!
          title: String!
        }

        # Incomplete fields
        type Query {
          "Search for blog posts using various criteria"
          searchPosts(
            "Filter criteria for blog posts"
            filter: PostFilter!
          ): [Post!]!
        }

        "Input type for filtering blog posts"
        input PostFilter {
          "Search by post title (case insensitive)"
          title: String
          "Filter posts by specific tags"
          tags: [String!]
          "Only return posts created after this date"
          createdAfter: String
        }
        "##);

        // Search for a term that appears in field descriptions
        let response = stream
            .call_tool("search", json!({"keywords": ["case insensitive"]}))
            .await;
        insta::assert_snapshot!(&response, @"");

        // Search for a term that appears in argument descriptions
        let response = stream.call_tool("search", json!({"keywords": ["criteria"]})).await;
        insta::assert_snapshot!(&response, @r##"
        # Incomplete fields
        type Query {
          "Search for blog posts using various criteria"
          searchPosts(
            "Filter criteria for blog posts"
            filter: PostFilter!
          ): [Post!]!
        }

        "Represents a blog post in the system"
        type Post {
          "The main content/body of the blog post"
          content: String!
          createdAt: String!
          id: ID!
          "List of tags associated with the post"
          tags: [String!]!
          title: String!
        }

        "Input type for filtering blog posts"
        input PostFilter {
          "Search by post title (case insensitive)"
          title: String
          "Filter posts by specific tags"
          tags: [String!]
          "Only return posts created after this date"
          createdAfter: String
        }
        "##);
    });
}
