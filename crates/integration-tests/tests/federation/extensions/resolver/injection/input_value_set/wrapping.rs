use crate::federation::extensions::resolver::validation::EchoExt;
use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn nullable_input_value_set() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(id: ID!): JSON @echo(input: null)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
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
                "input": null
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn nullable_input_value_set_not_provided() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(id: ID!): JSON @echo
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
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
              "directive": {},
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn list_of_input_value_set() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(a: String, b: String): JSON @echo(inputs: ["a", "b"])
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(inputs: [InputValueSet]!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(a: "A", b: "B") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "inputs": [
                  {
                    "a": "A"
                  },
                  {
                    "b": "B"
                  }
                ]
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
