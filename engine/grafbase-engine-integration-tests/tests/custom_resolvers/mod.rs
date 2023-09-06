use grafbase_engine_integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use grafbase_runtime::udf::{CustomResolverRequestPayload, CustomResolverResponse};
use serde_json::json;

#[test]
fn simple_custom_resolver() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                hello: String @resolver(name: "hello")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new().resolver("hello", |_| Ok(CustomResolverResponse::Success(json!("world")))),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { hello }").await.into_value(),
            @r###"
        {
          "data": {
            "hello": "world"
          }
        }
        "###
        );
    });
}

#[test]
fn nested_custom_resolver() {
    // Tests that errors inside list items propagate to the list item and not the list
    runtime().block_on(async {
        let schema = r#"
            type Query {
                list: [ObjectWithErrors]! @resolver(name: "list")
            }

            type ObjectWithErrors {
                item: Int! @resolver(name: "item")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", CustomResolverResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver("item", |payload: CustomResolverRequestPayload| {
                        Ok(CustomResolverResponse::Success(payload.parent.unwrap()["id"].clone()))
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { list { item }}",).await.into_value(),
            @r###"
        {
          "data": {
            "list": [
              {
                "item": 1
              },
              {
                "item": 2
              }
            ]
          }
        }
        "###
        );
    });
}
