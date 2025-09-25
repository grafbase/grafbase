use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn valid_big_int() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
                    @meta(value: 9223372036854775807)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: -923372036854775807)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                scalar BigInt

                directive @meta(value: BigInt!) on SCHEMA
                directive @echo(value: BigInt!) on FIELD_DEFINITION
            "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": 9223372036854775807
            }
          },
          "directive": {
            "value": -923372036854775807
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn float_to_big_int_coercion() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
                    @meta(value: 1.0)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: 7.0)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                scalar BigInt

                directive @meta(value: BigInt!) on SCHEMA
                directive @echo(value: BigInt!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": 1
            }
          },
          "directive": {
            "value": 7
          },
          "input": {}
        }
      }
    }
    "#);
}
