//! Tests of federation specific introspection

use integration_tests::{runtime, EngineBuilder, ResponseExt};
use serde_json::Value;

const TODO_SCHEMA: &str = r#"
    extend schema @federation(version: "2.3")

    type Todo @model {
        id: ID!
        title: String!
    }

    type User @key(fields: "id") {
        id: ID!
        name: String! @resolver(name: "user/name") @requires(fields: "id")
    }
"#;

#[test]
fn introspecting_service_field() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

        let data = engine.execute("query { _service { sdl } }").await.into_data::<Value>();
        let schema = data["_service"]["sdl"].as_str().unwrap();

        insta::assert_snapshot!(schema);
    });
}

#[test]
fn introspecting_service_field_when_no_federation() {
    runtime().block_on(async {
        let engine = EngineBuilder::new("type User @model { name: String }").build().await;

        let result = engine.execute("query { _service { sdl } }").await.into_value();

        insta::assert_json_snapshot!(result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unknown field \"_service\" on type \"Query\".",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ]
            }
          ]
        }
        "###);
    });
}
