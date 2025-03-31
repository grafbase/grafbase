use crate::federation::extensions::field_resolver::validation::EchoExt;
use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn multiple_fields() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "first limit filters { latest nested { id } }")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(first: 10, after: "79", filters: { latest: null, nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 10,
                  "filters": {
                    "latest": null,
                    "nested": {
                      "id": "78"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn all() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "*")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(first: 10, after: "79", filters: { latest: null, nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 10,
                  "after": "79",
                  "filters": {
                    "latest": null,
                    "nested": {
                      "id": "78",
                      "name": "Hi!"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn nested_all() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "first limit filters")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(first: 10, after: "79", filters: { latest: null, nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 10,
                  "filters": {
                    "latest": null,
                    "nested": {
                      "id": "78",
                      "name": "Hi!"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn default_values() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int = 100, limit: Int, after: String, filters: Filters): JSON @echo(input: "first limit filters { latest nested { id } }")
                }

                input Filters {
                    latest: Boolean = false
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(after: "79", filters: { nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 100,
                  "filters": {
                    "latest": false,
                    "nested": {
                      "id": "78"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn default_values_star() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int = 100, limit: Int, after: String, filters: Filters): JSON @echo(input: "*")
                }

                input Filters {
                    latest: Boolean = false
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(after: "79", filters: { nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 100,
                  "after": "79",
                  "filters": {
                    "latest": false,
                    "nested": {
                      "id": "78",
                      "name": "Hi!"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn extension_directive_default_value() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int = 100, limit: Int, after: String, filters: Filters): JSON @echo
                }

                input Filters {
                    latest: Boolean = false
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet! = "*") on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(after: "79", filters: { nested: { id: "78", name: "Hi!" } }) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "input": {
                  "first": 100,
                  "after": "79",
                  "filters": {
                    "latest": false,
                    "nested": {
                      "id": "78",
                      "name": "Hi!"
                    }
                  }
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
