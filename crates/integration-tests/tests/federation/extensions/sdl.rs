use std::{collections::HashMap, sync::Arc};

use engine::Engine;
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{
    error::PartialGraphqlError,
    extension::ExtensionDirective,
    hooks::{DynHookContext, EdgeDefinition},
};

#[derive(Default)]
struct EchoExt {
    sdl: &'static str,
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

    fn build(
        &self,
        schema_directives: Vec<ExtensionDirective<'_, serde_json::Value>>,
    ) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(EchoInstance {
            schema_directives: schema_directives
                .into_iter()
                .map(|dir| (dir.name.to_string(), dir.static_arguments))
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
        _field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        Ok(vec![
            Ok(serde_json::json!({
                "schema": &self.schema_directives,
                "directive": &directive.static_arguments,
            }));
            inputs.len()
        ])
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
