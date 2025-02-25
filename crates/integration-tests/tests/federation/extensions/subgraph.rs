use std::sync::Arc;

use engine::Engine;
use extension_catalog::Id;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension, TestExtensionBuilder, TestExtensionConfig},
    runtime,
};
use runtime::{error::PartialGraphqlError, extension::ExtensionFieldDirective, hooks::DynHookContext};
use serde_json::json;

use crate::federation::extensions::basic::GreetExt;

#[test]
fn extension_mixed_with_graphql_subgraph_root_fields() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type User {
                        name: String!
                    }

                    type Query {
                        user: User
                    }
                    "#,
                )
                .with_resolver("Query", "user", json!({"name": "Alice"}))
                .into_subgraph("x"),
            )
            .with_subgraph_sdl(
                "y",
                r#"
                    extend schema
                        @link(url: "greet-1.0.0", import: ["@greet"])

                    scalar JSON

                    type Query {
                        greet: JSON @greet
                    }

                "#,
            )
            .with_extension(GreetExt::with_sdl(
                r#"
                    extend schema @link(url: "http://specs.grafbase.com/grafbase")
                    directive @greet on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await
            .post("{ greet user { name } }")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greet": "Hi!",
            "user": {
              "name": "Alice"
            }
          }
        }
        "#);
    });
}

#[derive(Default)]
struct ResolveExt;

impl TestExtensionBuilder for ResolveExt {
    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: vec!["echo".to_string(), "echoArgs".to_string()],
            }),
            sdl: Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet"])
                scalar JSON
                directive @resolve(value: JSON, requires: FieldSet) on FIELD_DEFINITION
            "#,
            ),
        }
    }

    fn build(&self, _: Vec<(&str, serde_json::Value)>) -> Arc<dyn TestExtension> {
        Arc::new(ResolveInstance)
    }

    fn id(&self) -> extension_catalog::Id
    where
        Self: Sized,
    {
        Id {
            name: "resolve".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }
}

struct ResolveInstance;

#[async_trait::async_trait]
impl TestExtension for ResolveInstance {
    async fn resolve<'a>(
        &self,
        _context: &DynHookContext,
        directive: ExtensionFieldDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        Ok(inputs
            .into_iter()
            .map(|_| Ok(directive.arguments["value"].clone()))
            .collect())
    }
}

#[test]
fn nested_extension_in_same_subgraph() {
    runtime().block_on(async move {
        let response = Engine::builder()
            .with_subgraph_sdl(
                "y",
                r#"
                    extend schema
                        @link(url: "resolve-1.0.0", import: ["@resolve"])

                    type User {
                        name: String!
                        age: Int! @resolve(value: 892, requires: "name")
                    }

                    type Query {
                        user: User @resolve(value: { name: "Alice" })
                    }

                "#,
            )
            .with_extension(ResolveExt)
            .build()
            .await
            .post("{ user { name age } }")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "name": "Alice",
              "age": 892
            }
          }
        }
        "#);
    });
}
