use std::sync::Arc;

use engine::Engine;
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{error::PartialGraphqlError, extension::ExtensionFieldDirective, hooks::DynHookContext};

#[derive(Default)]
pub struct EchoJsonExt;

impl TestExtensionBuilder for EchoJsonExt {
    fn id(&self) -> Id {
        Id {
            name: "echo".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: vec!["echo".to_string()],
            }),
            sdl: Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["JsonTemplate"])

                directive @echo(data: JsonTemplate!) on FIELD_DEFINITION
                "#,
            ),
        }
    }

    fn build(&self, _: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(EchoJsonExt)
    }
}

#[async_trait::async_trait]
impl TestExtension for EchoJsonExt {
    async fn resolve<'a>(
        &self,
        _context: &DynHookContext,
        directive: ExtensionFieldDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        let data: serde_json::Value =
            serde_json::from_str(directive.arguments["data"].as_str().unwrap_or_default()).unwrap();
        Ok(vec![Ok(data.clone()); inputs.len()])
    }
}

#[test]
fn json_template() {
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
                    ): JSON @echo(data: """
                    {
                        "a": {{ args.a }},
                        "b": {{ args.b }},
                        "c": {{ args.c }},
                        "d": {{ args.d }},
                        "e": {{ args.e }},
                        "f": {{ args.f }}
                    }
                    """)
                }
                "#,
            )
            .with_extension(EchoJsonExt)
            .build()
            .await
            .post(r#"query { echo(a: 1, b: 2.7, c: false, d: "Hi!", e: "123890", f: "Bonjour" ) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "a": 1,
              "b": 2.7,
              "c": false,
              "d": "Hi!",
              "e": "123890",
              "f": "Bonjour"
            }
          }
        }
        "#);
    });
}

#[test]
fn json_should_escape_string_content() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: "{{ args.data }}")
                }
                "#,
            )
            .with_extension(EchoJsonExt)
            .build()
            .await
            .post(r#"query { echo(data: """{"test": "value"}""") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": "{\"test\": \"value\"}"
          }
        }
        "#);
    });
}

#[test]
fn json_should_render_objects_as_json() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: "{{ args.data }}")
                }
                "#,
            )
            .with_extension(EchoJsonExt)
            .build()
            .await
            .post(r#"query { echo(data: {name: "Alice",  pets: ["meow"]}) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "name": "Alice",
              "pets": [
                "meow"
              ]
            }
          }
        }
        "#);
    });
}

#[test]
fn json_should_render_lists_as_json() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: "{{ args.data }}")
                }
                "#,
            )
            .with_extension(EchoJsonExt)
            .build()
            .await
            .post(r#"query { echo(data: ["meow", {name: "Alice"}]) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": [
              "meow",
              {
                "name": "Alice"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn complex_template() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """[{{#args.data}} { "value":{{name}} } {{/args.data}}]""")
                }
                "#,
            )
            .with_extension(EchoJsonExt)
            .build()
            .await
            .post(r#"query { echo(data: [{name: "Alice"}]) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": []
          }
        }
        "#);
    });
}
