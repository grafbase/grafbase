use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::basic::GreetExt;

#[test]
fn extension_mixed_with_graphql_subgraph_root_fields() {
    runtime().block_on(async move {
        let response = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type User {
                        name: String!
                    }

                    type Query {
                        user: User
                    }
                    "#,
                )
                .with_resolver("Query", "user", json!({"name": "Alice"}))
                .into_subgraph("x"),
            )
            .with_subgraph_sdl(
                "y",
                r#"
                    extend schema
                        @link(url: "greet-1.0.0", import: ["@greet"])

                    scalar JSON

                    type Query {
                        greet: JSON @greet
                    }

                "#,
            )
            .with_extension(GreetExt::new().with_sdl(
                r#"
                    extend schema @link(url: "http://specs.grafbase.com/grafbase")
                    directive @greet on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post("{ greet user { name } }")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greet": "Hi!",
            "user": {
              "name": "Alice"
            }
          }
        }
        "#);
    });
}
