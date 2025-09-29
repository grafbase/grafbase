use graphql_mocks::{MockGraphQlServer, Subgraph, dynamic::DynamicSchema};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::resolver::ResolverExt;

const LIMIT_CONFIG: &str = r#"
[complexity_control]
mode = "enforce"
limit = 100
"#;

#[test]
fn test_uncomplex_query() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        engine.post("query { cheapField }").await
    });

    similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"cheapField": null}}));
}

#[test]
fn test_complex_query_while_off() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(ComplexitySchema).build().await;

        let response = engine.post("query { cheapField expensiveField  }").await;

        similar_asserts::assert_serde_eq!(
            response.body,
            serde_json::json!({"data": {"cheapField": null, "expensiveField": null}})
        );
    });
}

#[test]
fn complexity_control_should_work_with_virtual_subgraphs() {
    async fn build_with_complexity_limit(limit: usize) -> Gateway {
        Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])

                type Query {
                    super: SuperObj @resolve
                }

                type SuperObj {
                    obj: Obj
                }

                type Obj {
                    a: A
                }

                type A {
                    something: String @cost(weight: 5)
                    address: Address
                }

                type Address {
                    b: String @cost(weight: 10)
                    c: String @cost(weight: 5)
                }
                "#,
            )
            .with_extension(ResolverExt::json(json!({
                "obj": {
                    "a": {
                        "something": "hello",
                        "address": {
                            "b": "world",
                            "c": "!"
                        }
                    }
                }
            })))
            .with_toml_config(format!(
                r#"
                [complexity_control]
                mode = "enforce"
                limit = {limit}
                "#
            ))
            .build()
            .await
    }
    const QUERY: &str = "query { super { obj { a { something address { b c } } } } }";

    runtime().block_on(async move {
        let response = build_with_complexity_limit(23).await.post(QUERY).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = build_with_complexity_limit(24).await.post(QUERY).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "super": {
              "obj": {
                "a": {
                  "something": "hello",
                  "address": {
                    "b": "world",
                    "c": "!"
                  }
                }
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn test_complex_query_with_measure() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(
                r#"
                [complexity_control]
                mode = "measure"
                limit = 100
                "#,
            )
            .build()
            .await;

        let response = engine.post("query { cheapField expensiveField  }").await;

        similar_asserts::assert_serde_eq!(
            response.body,
            serde_json::json!({"data": {"cheapField": null, "expensiveField": null}})
        );
    });
}

#[test]
fn test_complex_query_with_enforce() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        let response = engine.post("query { expensiveField  }").await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"expensiveField": null}}));

        let response = engine.post("query { freeField, expensiveField  }").await;

        similar_asserts::assert_serde_eq!(
            response.body,
            serde_json::json!({"data": {"freeField": null, "expensiveField": null}})
        );

        let response = engine.post("query { cheapField expensiveField  }").await;

        insta::assert_json_snapshot!(response.body, @r###"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###)
    });
}

#[test]
fn test_assumed_list_size() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        let response = engine.post("query { sizedListField { blah }  }").await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"sizedListField": null}}));

        let response = engine.post("query { sizedListField { blah } cheapField }").await;

        insta::assert_json_snapshot!(response.body, @r###"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###)
    });
}

#[test]
fn test_sliced_list_size() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        let response = engine.post("query { slicingListField(first: 100) { blah }  }").await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"slicingListField": null}}));

        let response = engine.post("query { slicingListField(last: 100) { blah }  }").await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"slicingListField": null}}));

        let first_failed_response = engine.post("query { slicingListField(first: 101) { blah }  }").await;
        let last_failed_response = engine.post("query { slicingListField(last: 101) { blah }  }").await;

        similar_asserts::assert_serde_eq!(first_failed_response.body, last_failed_response.body);

        insta::assert_json_snapshot!(first_failed_response.body, @r###"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###)
    });
}

#[test]
fn test_require_one_slicing_argument() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        let response = engine.post("query { slicingListField { blah }  }").await;

        insta::assert_json_snapshot!(response.body, @r###"
        {
          "errors": [
            {
              "message": "Expected exactly one slicing argument on Query.slicingListField",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post("query { slicingListField(first: 10, last: 10) { blah }  }")
            .await;
        insta::assert_json_snapshot!(response.body, @r###"
        {
          "errors": [
            {
              "message": "Expected exactly one slicing argument on Query.slicingListField",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);

        let response = engine
            .post("query { slicingListFieldButNotRequiredArguments(first: 10, last: 10) { blah }  }")
            .await;

        similar_asserts::assert_serde_eq!(
            response.body,
            serde_json::json!({"data": {"slicingListFieldButNotRequiredArguments": null}})
        );

        let response = engine
            .post("query { slicingListFieldButNotRequiredArguments { blah }  }")
            .await;

        similar_asserts::assert_serde_eq!(
            response.body,
            serde_json::json!({"data": {"slicingListFieldButNotRequiredArguments": null}})
        );
    });
}

#[test]
fn test_sized_fields() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        let response = engine
            .post("query { connectionField(first: 50) { items { blah } }  }")
            .await;

        // Cost should be <100
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"connectionField": null}}));

        // We're not requesting the actual list field so this should be cheap
        let response = engine
            .post("query { connectionField(first: 100) { totalCount }  }")
            .await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"connectionField": null}}));

        // We're not requesting the actual list field so this should be cheap regardless of the big list
        // size
        let response = engine.post("query { connectionField(first: 200) { cursor }  }").await;

        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"connectionField": null}}));

        eprintln!("Running failed response");
        let failed_response_one = engine
            .post("query { connectionField(first: 200) { items { blah } }  }")
            .await;
        let failed_response_two = engine
            .post("query { connectionField(first: 90) { items { blah } totalCount } }")
            .await;

        similar_asserts::assert_serde_eq!(failed_response_one.body, failed_response_two.body);

        insta::assert_json_snapshot!(failed_response_one.body, @r###"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###)
    });
}

#[test]
fn test_argument_complexity() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(ComplexitySchema)
            .with_toml_config(LIMIT_CONFIG)
            .build()
            .await;

        // Cost should be way less than 100
        let response = engine.post("query { argumentsField(int: 1) { blah } }").await;
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"argumentsField": null}}));

        // Cost should be 100
        let response = engine.post("query { argumentsField(complexInt: 1) { blah }  }").await;
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"argumentsField": null}}));

        // Cost should be 100
        let response = engine
            .post("query { argumentsField(object: {int: 1}) { blah }  }")
            .await;
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"argumentsField": null}}));

        let response = engine
            .post("query { argumentsField(object: {complexInt: 1}) { blah }  }")
            .await;
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"argumentsField": null}}));

        // Cost should be > 100
        let first_fail = engine
            .post("query { argumentsField(otherInt: 1, object: {complexInt: 1}) { blah }  }")
            .await;

        let second_fail = engine
            .post("query { argumentsField(otherInt: 1, object: {nested: {complexInt: 1}}) { blah }  }")
            .await;

        let third_fail = engine
            .post("query { argumentsField(otherInt: 1, object: {nestedList: [{complexInt: 1}]}) { blah }  }")
            .await;

        insta::assert_json_snapshot!(first_fail.body, @r###"
        {
          "errors": [
            {
              "message": "Query exceeded complexity limit",
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);

        similar_asserts::assert_serde_eq!(first_fail.body, second_fail.body);
        similar_asserts::assert_serde_eq!(first_fail.body, third_fail.body);
    });
}

pub struct ComplexitySchema;

impl Subgraph for ComplexitySchema {
    fn name(&self) -> String {
        "complexity".into()
    }

    async fn start(self) -> MockGraphQlServer {
        let schema = DynamicSchema::builder(
            r#"
            type Query {
                freeField: String

                cheapField: String @cost(weight: 1)

                expensiveField: String @cost(weight: 100)

                sizedListField: [Item] @listSize(assumedSize: 100)

                slicingListField(first: Int, last: Int): [Item] @listSize(
                    slicingArguments: ["first", "last"]
                )

                slicingListFieldButNotRequiredArguments(first: Int, last: Int): [Item] @listSize(
                    slicingArguments: ["first", "last"]
                    requireOneSlicingArgument: false
                )

                connectionField(first: Int): Connection @listSize(
                    slicingArguments: "first"
                    sizedFields: "items"
                )

                argumentsField(
                    int: Int,
                    otherInt: Int @cost(weight: 2)
                    complexInt: Int @cost(weight: 98)
                    object: InputObject
                ): Item
            }

            input InputObject {
              int: Int
              complexInt: Int @cost(weight: 98)
              nested: InputObject
              nestedList: [InputObject]
            }

            type Connection {
                items: [Item]
                totalCount: Int @cost(weight: 50)
                cursor: String!
            }

            type Item {
                blah: String!
            }
            "#,
        )
        .finish();

        MockGraphQlServer::new(schema).await
    }
}
