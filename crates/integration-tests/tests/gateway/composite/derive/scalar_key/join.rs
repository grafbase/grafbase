use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn shareable_field() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive", "@shareable"])

                type Query {
                    product: Product
                }

                type Product {
                    id: ID!
                    code: String!
                    authorId: ID!
                    author: User! @derive
                }

                type User @key(fields: "id") {
                    id: ID!
                    category: ID @shareable
                }
                "#,
                )
                .with_resolver("Query", "product", json!({"authorId": "user_1"}))
                .into_subgraph("products"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        users: [User!]!
                    }

                    type User @key(fields: "id") {
                        id: ID!
                        category: ID @shareable
                    }
                    "#,
                )
                .with_entity_resolver("User", json!({"id": "user_1", "category": "cat1"}))
                .into_subgraph("users"),
            )
            .build()
            .await;

        let response = gateway.post("{ product { author { id category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "author": {
                "id": "user_1",
                "category": "cat1"
              }
            }
          }
        }
        "#
        );
    })
}

#[test]
fn external_field() {
    runtime().block_on(async {
        let gateway = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key", "@derive", "@external"])

                type Query {
                    product: Product
                }

                type Product {
                    id: ID!
                    code: String!
                    authorId: ID!
                    author: User! @derive
                }

                type User @key(fields: "id") {
                    id: ID!
                    category: ID @external
                }
                "#,
                )
                .with_resolver("Query", "product", json!({"authorId": "user_1"}))
                .into_subgraph("products"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        users: [User!]!
                    }

                    type User @key(fields: "id") {
                        id: ID!
                        category: ID
                    }
                    "#,
                )
                .with_entity_resolver("User", json!({"id": "user_1", "category": "cat1"}))
                .into_subgraph("users"),
            )
            .build()
            .await;

        let response = gateway.post("{ product { author { id category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "product": {
              "author": {
                "id": "user_1",
                "category": "cat1"
              }
            }
          }
        }
        "#
        );
    })
}
