use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn explicit_is() {
    runtime().block_on(async {
        let engine = Gateway::builder()
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
                authId: ID!
                author: User! @derive @is(field: "{ id: authId }")
            }

            type User @key(fields: "id") {
                id: ID!
            }
            "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "authId": "user_1"}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": {
                "id": "user_1"
              }
            }
          }
        }
        "#);
    })
}
