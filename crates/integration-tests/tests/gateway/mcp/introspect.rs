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

        insta::assert_snapshot!(&response, @r#"
        type User {
          id: ID!
          name: String!
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

        insta::assert_snapshot!(&response, @r#"
        union SearchResult = Comment | Post

        type Post {
          id: ID!
          title: String!
        }

        type Comment {
          id: ID!
          text: String!
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

        insta::assert_snapshot!(&response, @r#"
        interface Node {
          id: ID!
        }

        type Product implements Node {
          id: ID!
          price: Float!
        }

        type User implements Node {
          id: ID!
          name: String!
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

        insta::assert_snapshot!(&response, @r#"
        enum Status {
          DRAFT
          PUBLISHED
          ARCHIVED
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

        insta::assert_snapshot!(&response, @"scalar DateTime");
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

        insta::assert_snapshot!(&response, @r#"
        input SearchFilter {
          term: String!
          limit: Int
          sortBy: String
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

        insta::assert_snapshot!(&response, @r#"
        type User {
          id: ID!
          posts(first: Int = 10, offset: Int!, status: PostStatus = PUBLISHED): [Post!]!
        }

        enum PostStatus {
          DRAFT
          PUBLISHED
          ARCHIVED
        }

        type Post {
          id: ID!
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

        insta::assert_snapshot!(&response, @r#"
        input SearchFilterWithDefaults {
          term: String!
          limit: Int = 50
          sortBy: String = "createdAt"
          includeArchived: Boolean = false
        }
        "#);
    });
}

#[test]
fn should_show_mutations() {
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
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream.call_tool("introspect", json!({"types": ["Mutation"]})).await;

        insta::assert_snapshot!(&response, @r#"
        type Mutation {
          createUser: User
        }

        type User {
          id: ID!
          name: String!
        }
        "#);
    });
}

#[test]
fn test_descriptions() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                """
                A user in the system.
                """
                type User implements Node {
                    """
                    The unique identifier of the user.
                    """
                    id: ID!

                    """
                    The user's full name.
                    """
                    name: String!

                    """
                    Posts created by the user.
                    """
                    posts(
                        """
                        Maximum number of posts to return.
                        """
                        limit: Int = 10
                    ): [Post!]!
                }

                """
                A blog post written by a user.
                """
                type Post implements Node {
                    """
                    The unique identifier of the post.
                    """
                    id: ID!
                }

                """
                A comment on a post or another comment.
                """
                type Comment implements Node {
                    """
                    The unique identifier of the comment.
                    """
                    id: ID!

                    """
                    The content of the comment.
                    """
                    content: String!
                }

                """
                Interface for entities that can be uniquely identified.
                """
                interface Node {
                    """
                    The unique identifier of the node.
                    """
                    id: ID!
                }

                """
                Represents either a Post or a Comment in search results.
                """
                union SearchResult = Post | Comment

                """
                A custom date-time scalar that handles timestamps.
                """
                scalar DateTime

                """
                Input type for filtering posts.
                """
                input PostFilter {
                    """
                    Filter by author ID.
                    """
                    authorId: ID

                    "Filter by post status."
                    status: PostStatus = PUBLISHED

                    """
                    Filter by creation date.
                    """
                    createdAfter: DateTime
                }

                """
                The status of a post.
                I'm a multiline comment.
                """
                enum PostStatus {
                    """
                    Post is in draft state.
                    """
                    DRAFT

                    """
                    Post is published.
                    """
                    PUBLISHED
                }

                type Query {
                    user: User
                    node: Node
                    search: SearchResult
                    currentTime: DateTime
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

        let response = stream.call_tool("introspect", json!({"types": ["User", "Post", "Comment", "Node", "SearchResult", "DateTime", "PostFilter", "PostStatus"]})).await;

        insta::assert_snapshot!(&response, @r#"
        "A comment on a post or another comment."
        type Comment implements Node {
          "The content of the comment."
          content: String!
          "The unique identifier of the comment."
          id: ID!
        }

        "Represents either a Post or a Comment in search results."
        union SearchResult = Comment | Post

        "A custom date-time scalar that handles timestamps."
        scalar DateTime

        "A blog post written by a user."
        type Post implements Node {
          "The unique identifier of the post."
          id: ID!
        }

        "A user in the system."
        type User implements Node {
          "The unique identifier of the user."
          id: ID!
          "The user's full name."
          name: String!
          "Posts created by the user."
          posts(
            "Maximum number of posts to return."
            limit: Int = 10
          ): [Post!]!
        }

        "Interface for entities that can be uniquely identified."
        interface Node {
          "The unique identifier of the node."
          id: ID!
        }

        "Input type for filtering posts."
        input PostFilter {
          "Filter by author ID."
          authorId: ID
          "Filter by post status."
          status: PostStatus = PUBLISHED
          "Filter by creation date."
          createdAfter: DateTime
        }

        """
        The status of a post.
        I'm a multiline comment.
        """
        enum PostStatus {
          DRAFT
          PUBLISHED
        }
        "#);
    });
}
