use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn default_values() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput! = { x: 3 }, b: Int): JSON @echoArgs
                    echo: JSON @echo(fields: "field")
                }

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "field": {
            "input": {
              "x": 3,
              "y": "default"
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn default_values_partial_override() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput! = { x: 3 }, b: Int): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: { x: 5 })")
                }

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "field": {
            "input": {
              "x": 5,
              "y": "default"
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn default_values_override() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput! = { x: 3 }, b: Int): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: { x: 5, y: \"override\" })")
                }

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "field": {
            "input": {
              "x": 5,
              "y": "override"
            }
          }
        }
      }
    }
    "#);
}
