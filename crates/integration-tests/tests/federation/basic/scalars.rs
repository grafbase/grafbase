use engine::Engine;
use graphql_mocks::{dynamic::DynamicSchema, AlmostEmptySchema, FakeGithubSchema};
use integration_tests::{federation::EngineExt, runtime};
use serde_json::json;

#[test]
fn should_not_raise_an_error_on_null_for_required_json() {
    runtime().block_on(async move {
        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    scalar JSON

                    type Query {
                        node: JSON!
                    }
                    "#,
                )
                .with_resolver("Query", "node", json!({"name": "Alice", "id": null}))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = gateway.post("{ node }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": "Alice",
              "id": null
            }
          }
        }
        "#);
    })
}

#[test]
fn supports_custom_scalars() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine.post("query { favoriteRepository }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "favoriteRepository": {
          "owner": "rust-lang",
          "name": "rust"
        }
      }
    }
    "###);
}

#[test]
fn supports_unused_builtin_scalars() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(AlmostEmptySchema).build().await;

        engine
            .post("query Blah($id: ID!) { string(input: $id) }")
            .variables(json!({"id": "1"}))
            .await
    });

    // Bit of a poor test this because we can never pass a valid query that makes use of a scalar that doesn't exist.
    // But so long as any errors below don't include "Unknown type `ID` or similar I think we're good"

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Variable $id doesn't have the right type. Declared as 'ID!' but used as 'String!'",
          "locations": [
            {
              "line": 1,
              "column": 38
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "###);
}

#[test]
fn coerces_ints_to_floats() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Float): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine.post("query { foo(input: 1) }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "foo": 1.0
      }
    }
    "#);
}

#[test]
fn coerces_floats_to_ints_where_possible() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Int): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine.post("query { foo(input: 1.0) }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "foo": 1.0
      }
    }
    "#);
}

#[test]
fn refuses_to_lose_precision_when_converting_floats_to_ints() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Int): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine.post("query { foo(input: 1.5) }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "Found a Float value where we expected a Int scalar",
          "locations": [
            {
              "line": 1,
              "column": 20
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn coerces_variable_ints_to_floats() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Float): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine
            .post("query($foo: Float) { foo(input: $foo) }")
            .variables(json!({"foo": 1}))
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "foo": 1.0
      }
    }
    "#);
}

#[test]
fn coerces_variable_floats_to_ints_where_possible() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Int): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine
            .post("query($foo: Int) { foo(input: $foo) }")
            .variables(json!({"foo": 1.0}))
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "foo": 1.0
      }
    }
    "#);
}

#[test]
fn refuses_to_lose_precision_when_converting_variable_floats_to_ints() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                        type Query {
                          foo(input: Int): Float
                        }
                    "#,
                )
                .with_resolver("Query", "foo", json!(1.0))
                .into_subgraph("foo"),
            )
            .build()
            .await;

        engine
            .post("query($foo: Int) { foo(input: $foo) }")
            .variables(json!({"foo": 1.5}))
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "Variable $foo has an invalid value. Found value 1.5 which cannot be coerced into a Int scalar",
          "locations": [
            {
              "line": 1,
              "column": 7
            }
          ],
          "extensions": {
            "code": "VARIABLE_ERROR"
          }
        }
      ]
    }
    "#);
}
