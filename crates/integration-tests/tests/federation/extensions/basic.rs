use std::sync::Arc;

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

#[derive(Default, Clone)]
struct Ext;

impl TestExtensionBuilder for Ext {
    fn id() -> Id {
        Id {
            name: "test".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config() -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: vec!["rest".to_string()],
            }),
            sdl: Some(
                r#"
                directive @greet on FIELD_DEFINITION
                "#,
            ),
        }
    }

    fn build(
        &self,
        _schema_directives: Vec<ExtensionDirective<'_, serde_json::Value>>,
    ) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(Ext)
    }
}

#[async_trait::async_trait]
impl TestExtension for Ext {
    async fn resolve<'a>(
        &self,
        _context: &DynHookContext,
        _field: EdgeDefinition<'a>,
        _directive: ExtensionDirective<'a, serde_json::Value>,
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
                    REST @extension__link(url: "test-1.0.0")
                }

                enum join__Graph {
                    A @join__graph(name: "a")
                }

                extend type Query {
                    greeting(name: String): String @extension__directive(graph: A, extension: REST, name: "greet")
                }
                "#,
            )
            .with_extension::<Ext>()
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
                    @link(url: "test-1.0.0", import: ["@greet"])

                type Query {
                    greeting(name: String): String @greet
                }
                "#,
            )
            .with_extension::<Ext>()
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
