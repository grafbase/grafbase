use std::sync::Arc;

use engine::Engine;
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{error::PartialGraphqlError, extension::ExtensionFieldDirective};

#[derive(Default, Clone)]
pub struct GreetExt {
    sdl: Option<&'static str>,
}

impl GreetExt {
    pub fn with_sdl(sdl: &'static str) -> Self {
        Self { sdl: Some(sdl) }
    }
}

impl TestExtensionBuilder for GreetExt {
    fn id(&self) -> Id {
        Id {
            name: "greet".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: None,
            }),
            sdl: self.sdl.or(Some(
                r#"
                directive @greet on FIELD_DEFINITION
                "#,
            )),
        }
    }

    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(GreetExt::default())
    }
}

#[async_trait::async_trait]
impl TestExtension for GreetExt {
    async fn resolve_field(
        &self,
        _headers: http::HeaderMap,
        _directive: ExtensionFieldDirective<'_, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        Ok(vec![Ok(serde_json::json!("Hi!")); inputs.len()])
    }
}

#[test]
fn simple_resolver_from_federated_sdl() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_federated_sdl(
                r#"
                enum extension__Link {
                    REST @extension__link(url: "greet-1.0.0")
                }

                enum join__Graph {
                    A @join__graph(name: "a")
                }

                extend type Query {
                    greeting(name: String): String @extension__directive(graph: A, extension: REST, name: "greet")
                }
                "#,
            )
            .with_extension(GreetExt::default())
            .build()
            .await;

        engine.post("query { greeting }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": "Hi!"
      }
    }
    "#);
}

#[test]
fn simple_resolver_from_subgraph_sdl() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "greet-1.0.0", import: ["@greet"])

                type Query {
                    greeting(name: String): String @greet
                }
                "#,
            )
            .with_extension(GreetExt::default())
            .build()
            .await;

        engine.post("query { greeting }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": "Hi!"
      }
    }
    "#);
}
