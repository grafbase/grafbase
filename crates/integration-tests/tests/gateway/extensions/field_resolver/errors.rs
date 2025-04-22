use engine::GraphqlError;
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

#[test]
fn item_error_on_required_field() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test: JSON! @resolve
                }
                "#,
            )
            .with_extension(StaticFieldResolverExt::item_error(GraphqlError::internal_server_error()))
            .build()
            .await;

        engine.post("query { test }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "Internal server error",
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
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn item_error_on_nullable_field() {
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
            .with_extension(StaticFieldResolverExt::item_error(GraphqlError::internal_server_error()))
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
          "message": "Internal server error",
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
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn resolver_error_on_required_field() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "static-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test: JSON! @resolve
                }
                "#,
            )
            .with_extension(StaticFieldResolverExt::resolver_error(
                GraphqlError::internal_server_error(),
            ))
            .build()
            .await;

        engine.post("query { test }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": null,
      "errors": [
        {
          "message": "Internal server error",
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
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn resolver_error_on_nullable_field() {
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
            .with_extension(StaticFieldResolverExt::resolver_error(
                GraphqlError::internal_server_error(),
            ))
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
          "message": "Internal server error",
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
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "#);
}
