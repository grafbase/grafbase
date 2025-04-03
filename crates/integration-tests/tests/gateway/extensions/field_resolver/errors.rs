use integration_tests::{gateway::Gateway, runtime};

use crate::gateway::extensions::field_resolver::StaticFieldResolverExt;

#[test]
fn invalid_json() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test: JSON @resolve
                }
                "#,
            )
            .with_extension(StaticFieldResolverExt::json("{/}".into()))
            .build()
            .await;

        engine.post("query { test }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "test": null
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "test"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}
