use async_graphql::dynamic::ResolverContext;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn validate_query_argument() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query { 
                        test(input: TestInput): JSON
                    }

                    scalar JSON

                    input TestInput @oneOf {
                        a: Int
                        b: String
                    }
                "#,
                )
                .with_resolver("Query", "test", |ctx: ResolverContext<'_>| {
                    serde_json::to_value(ctx.args.get("input").unwrap().as_value()).ok()
                })
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine.post("query { test(input: { a: 1 })}").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "a": 1
            }
          }
        }
        "#);

        let response = engine.post(r#"query { test(input: { b: "2" })}"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "b": "2"
            }
          }
        }
        "#);

        let response = engine.post(r#"query { test(input: {})}"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Exactly one field must be provided for TestInput with @oneOf: No field was provided",
              "locations": [
                {
                  "line": 1,
                  "column": 21
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine.post(r#"query { test(input: { a: 1, b: "2"})}"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Exactly one field must be provided for TestInput with @oneOf: 2 fields (a,b) were provided",
              "locations": [
                {
                  "line": 1,
                  "column": 21
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn validate_variable() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query { 
                        test(input: TestInput): JSON
                    }

                    scalar JSON

                    input TestInput @oneOf {
                        a: Int
                        b: String
                    }
                "#,
                )
                .with_resolver("Query", "test", |ctx: ResolverContext<'_>| {
                    serde_json::to_value(ctx.args.get("input").unwrap().as_value()).ok()
                })
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine
            .post("query($a: Int!) { test(input: { a: $a })}")
            .variables(json!({"a": 1}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "a": 1
            }
          }
        }
        "#);

        let response = engine
            .post(r#"query($b: String!) { test(input: { b: $b })}"#)
            .variables(json!({"b": "2"}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "b": "2"
            }
          }
        }
        "#);

        let response = engine.post(r#"query($a: Int) { test(input: { a: $a })}"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Variable $a is used for the field 'a' of TestInput with @oneOf and thus must be provided",
              "locations": [
                {
                  "line": 1,
                  "column": 35
                }
              ],
              "extensions": {
                "code": "VARIABLE_ERROR"
              }
            }
          ]
        }
        "#);

        let response = engine
            .post(r#"query($a: Int) { test(input: { a: $a })}"#)
            .variables(json!({"a": null}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Variable $a is used for the field 'a' of TestInput with @oneOf and thus must not be null",
              "locations": [
                {
                  "line": 1,
                  "column": 35
                }
              ],
              "extensions": {
                "code": "VARIABLE_ERROR"
              }
            }
          ]
        }
        "#);

        // Not supported today as that would need extra validation at runtime to detect if both a &
        // b are used together.
        let response = engine
            .post(r#"query($a: Int, $b: String) { test(input: { a: $a, b: $b })}"#)
            .variables(json!({"a": 1}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Exactly one field must be provided for TestInput with @oneOf: 2 fields (a,b) were provided",
              "locations": [
                {
                  "line": 1,
                  "column": 42
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn validate_default_argument() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query { 
                    test(input: TestInput = { a: 1 }): JSON
                }

                scalar JSON

                input TestInput @oneOf {
                    a: Int
                    b: String
                }
                "#,
            )
            .try_build()
            .await;
        insta::assert_debug_snapshot!(result.err(), @"None");

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query { 
                    test(input: TestInput = { b: "1" }): JSON
                }

                scalar JSON

                input TestInput @oneOf {
                    a: Int
                    b: String
                }
                "#,
            )
            .try_build()
            .await;
        insta::assert_debug_snapshot!(result.err(), @"None");

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query { 
                    test(input: TestInput = {}): JSON
                }

                scalar JSON

                input TestInput @oneOf {
                    a: Int
                    b: String
                }
                "#,
            )
            .try_build()
            .await;
        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.test.input, found an invalid default value: Exactly one field must be provided for TestInput with @oneOf: No field was provided at path '.input'. See schema at 19:27:\n{}",
        )
        "#);

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "x",
                r#"
                type Query { 
                    test(input: TestInput = { a: 1, b: "1"}): JSON
                }

                scalar JSON

                input TestInput @oneOf {
                    a: Int
                    b: String
                }
                "#,
            )
            .try_build()
            .await;
        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.test.input, found an invalid default value: Exactly one field must be provided for TestInput with @oneOf: 2 fields (a,b) were provided at path '.input'. See schema at 19:27:\n{a: 1, b: \"1\"}",
        )
        "#);
    })
}
