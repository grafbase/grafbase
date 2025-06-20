use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

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
                    post: Post
                }

                type Post {
                    id: ID!
                    code: String!
                    authorId: ID!
                    authorX: ID!
                    author: User! @derive
                }

                type User @key(fields: "id x") {
                    id: ID!
                    x: ID!
                    category: ID @external
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "post",
                    json!({"id": "post_1", "authorId": "user_1", "authorX": "user_x"}),
                )
                .into_subgraph("posts"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        users: [User!]!
                    }

                    type User @key(fields: "id x") {
                        id: ID!
                        x: ID!
                        category: ID
                    }
                    "#,
                )
                .with_entity_resolver("User", json!({"id": "user_1", "category": "cat1"}))
                .into_subgraph("users"),
            )
            .build()
            .await;

        let response = gateway.post("{ post { author { id category } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
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
