use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
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
            .with_custom_resolvers(RustUdfs::new().resolver("hello", |_| Ok(UdfResponse::Success(json!("world")))))
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
    // Tests that you can nest a custom resolver inside a custom resolver
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
                    .resolver("list", UdfResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver("item", |payload: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(payload.parent.unwrap()["id"].clone()))
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

#[test]
fn custom_resolver_context() {
    runtime().block_on(async {
        let schema = r#"
            type Query {
                list: [Object]! @resolver(name: "list")
            }

            type Object {
                item: JSON! @resolver(name: "item")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", UdfResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver("item", |payload: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(payload.info.unwrap()))
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
                "item": {
                  "fieldName": "item",
                  "path": {
                    "key": "item",
                    "prev": {
                      "key": 0,
                      "prev": {
                        "key": "list",
                        "prev": null,
                        "typename": "[Object]!"
                      },
                      "typename": "Object"
                    },
                    "typename": "JSON!"
                  },
                  "variableValues": {}
                }
              },
              {
                "item": {
                  "fieldName": "item",
                  "path": {
                    "key": "item",
                    "prev": {
                      "key": 1,
                      "prev": {
                        "key": "list",
                        "prev": null,
                        "typename": "[Object]!"
                      },
                      "typename": "Object"
                    },
                    "typename": "JSON!"
                  },
                  "variableValues": {}
                }
              }
            ]
          }
        }
        "###
        );
    });
}

#[test]
fn custom_resolver_with_interfaces() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                foos: [Foo!]! @resolver(name: "foos")
            }

            interface Foo {
                name: String
            }

            type Bar implements Foo {
                name: String!
                barField: String!
            }

            type Baz implements Foo {
                name: String!
                bazField: String! @resolver(name: "bazField")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver(
                        "foos",
                        UdfResponse::Success(json!([
                            {"__typename": "Bar", "name": "A Bar", "barField": "Bloop"},
                            {"__typename": "Baz", "name": "Bazza"},
                        ])),
                    )
                    .resolver(
                        "bazField",
                        UdfResponse::Success(json!("hello from the bazField resolver")),
                    ),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute(r#"
                query {
                    foos {
                        name
                        ... on Bar {
                            barField
                        }
                        ... on Baz {
                            bazField
                        }
                    }
                }
            "#).await.into_value(),
            @r###"
        {
          "data": {
            "foos": [
              {
                "name": "A Bar",
                "barField": "Bloop"
              },
              {
                "name": "Bazza",
                "bazField": "hello from the bazField resolver"
              }
            ]
          }
        }
        "###
        );
    });
}
