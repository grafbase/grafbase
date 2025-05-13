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
                    cmIds: [ID!]!
                    comments: [Comment!]! @derive @is(field: "cmIds[{ id: . }]")
                }

                type Comment @key(fields: "id") {
                    id: ID!
                }
                "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1", "cmIds": ["c1", "c2"]}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { post { id comments { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "comments": [
                {
                  "id": "c1"
                },
                {
                  "id": "c2"
                }
              ]
            }
          }
        }
        "#);
    })
}
