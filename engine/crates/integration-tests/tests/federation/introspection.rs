//! Tests of federation specific introspection

use integration_tests::{runtime, EngineBuilder, ResponseExt};
use serde_json::Value;

const TODO_SCHEMA: &str = r#"
    extend schema @federation(version: "2.3")

    type Todo @model {
        id: ID!
        title: String!
    }
"#;

#[test]
fn introspecting_service_field() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).with_local_dynamo().build().await;

        let data = engine.execute("query { _service { sdl } }").await.into_data::<Value>();
        let schema = data["_service"]["sdl"].as_str().unwrap();

        insta::assert_snapshot!(schema);
    });
}

#[test]
fn introspecting_service_field_when_no_federation() {
    runtime().block_on(async {
        let engine = EngineBuilder::new("type User @model { name: String }")
            .with_local_dynamo()
            .build()
            .await;

        let result = engine.execute("query { _service { sdl } }").await.into_value();

        insta::assert_json_snapshot!(result, @r###"
        {
          "data": null,
          "errors": [
            {
              "locations": [
                {
                  "column": 9,
                  "line": 1
                }
              ],
              "message": "Unknown field \"_service\" on type \"Query\"."
            }
          ]
        }
        "###);
    });
}
