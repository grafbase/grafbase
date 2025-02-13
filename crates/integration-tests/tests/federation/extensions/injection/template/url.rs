use crate::federation::extensions::validation::EchoExt;
use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn url_template() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(
                        a: Int!
                        b: Float!
                        c: Boolean!
                        d: String!
                        e: ID!
                        f: JSON!
                    ): JSON @echo(url: "https://example.com/echo/?a={{ args.a }}&b={{ args.b }}&c={{ args.c }}&d={{ args.d }}&e={{ args.e }}&f={{ args.f }}")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["UrlTemplate"])

                directive @echo(url: UrlTemplate!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(a: 1, b: 2.7, c: false, d: "Hi!", e: "123890", f: "Bonjour" ) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "url": "https://example.com/echo/?a=1&b=2.7&c=false&d=Hi%21&e=123890&f=Bonjour"
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn url_template_should_escape_strings() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(query: String!): JSON @echo(url: "https://example.com/echo/?q={{ args.query }}")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["UrlTemplate"])

                directive @echo(url: UrlTemplate!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(query: "café != thé, non?") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "url": "https://example.com/echo/?q=caf%C3%A9%20%21%3D%20th%C3%A9%2C%20non%3F"
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn url_template_should_encode_objects_as_json_and_then_escape() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON!): JSON @echo(url: "https://example.com/echo/?data={{ args.data }}")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["UrlTemplate"])

                directive @echo(url: UrlTemplate!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(data: {key: "value"}) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "url": "https://example.com/echo/?data=%7B%22key%22%3A%22value%22%7D"
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn url_template_should_encode_lists_as_json_and_then_escape() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON!): JSON @echo(url: "https://example.com/echo/?data={{ args.data }}")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["UrlTemplate"])

                directive @echo(url: UrlTemplate!) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post(r#"query { echo(data: [1, "value"]) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {},
              "directive": {
                "url": "https://example.com/echo/?data=%5B1%2C%22value%22%5D"
              }
            }
          }
        }
        "#);
    });
}
