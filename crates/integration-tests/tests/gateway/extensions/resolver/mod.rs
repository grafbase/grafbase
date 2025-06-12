mod subscription;

use integration_tests::{gateway::Gateway, runtime};

#[test]
fn wasm_basic() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-17-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test(input: String): JSON @resolve(data: {value: 1})
                }
                "#,
            )
            .with_extension("resolver-17")
            .build()
            .await;

        let response = engine.post("query { test(input: \"hi!\") }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "args": {
                "input": "hi!"
              },
              "config": {
                "key": null
              },
              "directive": {
                "data": {
                  "value": 1
                }
              }
            }
          }
        }
        "#);
    })
}
