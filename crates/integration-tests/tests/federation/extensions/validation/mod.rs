mod default_value;
mod definition;
mod enum_;
mod fields;
mod list;
mod location;
mod non_null;
mod scalar;

use std::{collections::HashMap, sync::Arc};

use engine::Engine;
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{error::PartialGraphqlError, extension::ExtensionFieldDirective, hooks::DynHookContext};

#[derive(Default)]
pub struct EchoExt {
    pub sdl: &'static str,
}

impl EchoExt {
    pub fn with_sdl(sdl: &'static str) -> Self {
        Self { sdl }
    }
}

impl TestExtensionBuilder for EchoExt {
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
            sdl: Some(self.sdl),
        }
    }

    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(EchoInstance {
            schema_directives: schema_directives
                .into_iter()
                .map(|(name, args)| (name.to_string(), args))
                .collect(),
        })
    }
}

struct EchoInstance {
    schema_directives: HashMap<String, serde_json::Value>,
}

#[async_trait::async_trait]
impl TestExtension for EchoInstance {
    async fn resolve<'a>(
        &self,
        _context: &DynHookContext,
        directive: ExtensionFieldDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        let json = serde_json::json!({
            "schema": self.schema_directives,
            "directive": directive.arguments,
        });
        Ok(vec![Ok(json.clone()); inputs.len()])
    }
}

#[test]
fn simple_echo() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: "meta")

                scalar JSON

                type Query {
                    echo: JSON @echo(value: "something")
                }
                "#,
            )
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            })
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": "meta"
            }
          },
          "directive": {
            "value": "something"
          }
        }
      }
    }
    "#);
}

#[test]
fn too_many_arguments() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                scalar JSON

                type Query {
                    echo: JSON @echo(value: "ste", other: 1)
                }
                "#,
            )
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Unknown argumant named 'other'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: "str", other: 1)

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive @meta: Unknown argumant named 'other'",
        )
        "#);
    });
}
