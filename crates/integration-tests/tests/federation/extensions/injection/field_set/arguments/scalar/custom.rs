use engine::Engine;

use crate::federation::extensions::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn valid_any() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

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
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

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
