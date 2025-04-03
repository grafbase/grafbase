mod selection_set;
mod validation;
mod wrapping;

use crate::federation::extensions::field_resolver::validation::EchoExt;
use integration_tests::{federation::Gateway, runtime};

#[test]
fn basic_input_value_set() {
    runtime().block_on(async move {
        let response = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(id: ID!): JSON @echo(input: "id")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(id: "1") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "id": "1"
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
