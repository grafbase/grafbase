use std::sync::Arc;

use engine::{Engine, GraphqlError};
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestManifest},
    runtime,
};
use runtime::extension::Data;

#[derive(Default)]
pub struct EchoJsonDataExt;

impl TestExtensionBuilder for EchoJsonDataExt {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: "echo".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                resolver_directives: Some(vec!["echo".to_string()]),
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
        Arc::new(EchoJsonDataExt)
    }
}

#[async_trait::async_trait]
impl TestExtension for EchoJsonDataExt {
    async fn resolve_field(
        &self,
        _directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        let data = directive_arguments["data"].as_str().unwrap_or_default();
        Ok(vec![Ok(Data::JsonBytes(data.as_bytes().to_vec())); inputs.len()])
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
            .with_extension(EchoJsonDataExt)
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
            .with_extension(EchoJsonDataExt)
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
            .with_extension(EchoJsonDataExt)
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
            .with_extension(EchoJsonDataExt)
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
fn iterate_object_within_list() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """[{{#args.data}} { "value":{{name}} } {{^-last}},{{/-last}} {{/args.data}}]""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: [{name: "Alice"}, {name: "Bob"}]) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": [
              {
                "value": "Alice"
              },
              {
                "value": "Bob"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn iterate_string_list() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """[{{#args.data}} { "value": {{.}} } {{^-last}},{{/-last}} {{/args.data}}]""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: ["Alice", "Bob"]) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": [
              {
                "value": "Alice"
              },
              {
                "value": "Bob"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn object_section() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """{{#args.data}}{ "a": {{name}}, "b": {{friend}} }{{/args.data}}""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: {name: "Alice", friend: "Bob"}) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "a": "Alice",
              "b": "Bob"
            }
          }
        }
        "#);
    });
}

#[test]
fn string_section() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """{{#args.data}}{{.}}{{/args.data}}""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: "something") }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "args": {
                "data": "something"
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn null_section() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """{{#args.data}}{{.}}{{/args.data}}""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: null) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "args": {
                "data": null
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn int_section() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """{{#args.data}}{{.}}{{/args.data}}""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: 1) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "args": {
                "data": 1
              }
            }
          }
        }
        "#);
    });
}

#[test]
fn boolean_section() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(data: JSON): JSON @echo(data: """{{#args.data}}{{.}}{{/args.data}}""")
                }
                "#,
            )
            .with_extension(EchoJsonDataExt)
            .build()
            .await
            .post(r#"query { echo(data: true) }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "args": {
                "data": true
              }
            }
          }
        }
        "#);
    });
}
