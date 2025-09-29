use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn valid_any() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON
                scalar Custom

                type Query {
                    field(value: Custom!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: {x: 7.123, y: null, z: false, w: VALUE})")
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
            "value": {
              "w": "VALUE",
              "x": 7.123,
              "y": null,
              "z": false
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn valid_any_array() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON
                scalar Custom

                type Query {
                    field(value: Custom!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: [{a:1}, 789, \"test\"])")
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
            "value": [
              {
                "a": 1
              },
              789,
              "test"
            ]
          }
        }
      }
    }
    "#);
}
