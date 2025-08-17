use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn invalid_key_field_type() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                authorId: Int!
                author: User! @derive
            }

            type User @key(fields: "id") {
                id: ID!
            }
            "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "author_i_D": "user_1"}))
                .into_subgraph("x"),
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Post.author, for directive @composite__derive: Derived field must match at least one @key
        See schema at 18:35:
        (graph: X)
        "#);
    })
}

#[test]
fn incompatible_key_field_wrapping() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    author_id: [ID!]!
                    author: User! @derive
                }

                type User {
                    id: ID!
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Product.author, for directive @composite__derive: Type User doesn't define any keys with @key directive that may be used for @derive
        See schema at 24:1:
        type User
          @join__type(graph: EXT)
        {
          id: ID!
        }
        "#);
    })
}

#[test]
fn missing_key_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
            extend schema
                @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@derive", "@key", "@is"])

            type Query {
                post: Post!
            }

            type Post {
                id: ID!
                authorId: ID!
                author: User! @derive
            }

            type User @key(fields: "id x") {
                id: ID!
                x: ID!
            }
            "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "author_i_D": "user_1"}))
                .into_subgraph("x"),
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Post.author, for directive @composite__derive: Derived field must match at least one @key
        See schema at 18:35:
        (graph: X)
        "#);
    })
}

#[test]
fn missing_field() {
    runtime().block_on(async {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive"])

                type Query {
                    productBatch(ids: [ID!]!): [Product!]!
                }

                type Product {
                    id: ID!
                    code: String!
                    authorId: ID!
                    author: User! @derive
                }

                type User @key(fields: "id") {
                    id: ID!
                    category: ID
                }
                "#,
            )
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r"
        At site Product.author, for directive @composite__derive: Field User.category is unprovidable for this @derive
        See schema at 18:17:
        @composite__derive
        ");
    })
}
