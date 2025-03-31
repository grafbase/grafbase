use std::sync::Arc;

use engine::{Engine, GraphqlError};
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestManifest, json_data},
    runtime,
};
use runtime::extension::Data;

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
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: "greet".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
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
        _directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Ok(vec![Ok(json_data("Hi!")); inputs.len()])
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
