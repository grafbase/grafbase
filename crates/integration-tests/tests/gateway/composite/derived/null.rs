use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};
use serde_json::json;

use crate::gateway::extensions::authorization::DenySites;

#[test]
fn null_entity() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    extend schema
                        @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@is"])

                    type Query {
                        post: Post!
                    }

                    type Post {
                        id: ID!
                        author_id: ID
                        author: User @is(field: "{ id: author_id }")
                    }

                    type User {
                        id: ID
                    }
                "#,
                )
                .with_resolver("Query", "post", json!({"id": "post_1"}))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["User.id"])))
            .build()
            .await;

        let response = engine.post("query { post { id author { id } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "post": {
              "id": "post_1",
              "author": null
            }
          }
        }
        "#);
    })
}
