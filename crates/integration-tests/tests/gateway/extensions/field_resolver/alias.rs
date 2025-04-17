use crate::gateway::extensions::field_resolver::validation::EchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn root_field_alias() {
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
            .post(r#"query { a: echo(id: "a") b: echo(id: "b") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "a": {
              "schema": {},
              "directive": {
                "input": {
                  "id": "a"
                }
              },
              "input": {}
            },
            "b": {
              "schema": {},
              "directive": {
                "input": {
                  "id": "b"
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
