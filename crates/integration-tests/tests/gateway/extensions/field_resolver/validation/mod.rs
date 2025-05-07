mod default_value;
mod definition;
mod enum_;
mod fields;
mod list;
mod location;
mod non_null;
mod one_of;
mod scalar;

use std::{collections::HashMap, sync::Arc};

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::{
    gateway::{
        AnyExtension, FieldResolverTestExtension, FieldResolverTestExtensionBuilder, Gateway, TestManifest, json_data,
    },
    runtime,
};
use runtime::extension::Data;

#[derive(Default, Clone, Copy)]
pub struct EchoExt {
    pub sdl: &'static str,
}

impl EchoExt {
    pub fn with_sdl(sdl: &'static str) -> Self {
        Self { sdl }
    }
}

impl AnyExtension for EchoExt {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "echo".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::FieldResolver(extension_catalog::FieldResolverType {
                resolver_directives: Some(vec!["echo".to_string()]),
            }),
            sdl: Some(self.sdl),
        });
        state.test.field_resolver_builders.insert(id, Arc::new(self));
    }
}

impl FieldResolverTestExtensionBuilder for EchoExt {
    fn build(
        &self,
        schema_directives: Vec<(&str, serde_json::Value)>,
    ) -> std::sync::Arc<dyn FieldResolverTestExtension> {
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
impl FieldResolverTestExtension for EchoInstance {
    async fn resolve_field(
        &self,
        _directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Ok(inputs
            .into_iter()
            .map(|input| {
                Ok(json_data(serde_json::json!({
                    "schema": self.schema_directives,
                    "directive": directive_arguments,
                    "input": input
                })))
            })
            .collect())
    }
}

#[test]
fn simple_echo() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
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
            .with_extension(EchoExt::with_sdl(
                r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            ))
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
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn too_many_arguments() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Gateway::builder()
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
            .with_extension(EchoExt::with_sdl(
                r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown argumant named 'other'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: "ste", other: 1})
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
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
            .with_extension(EchoExt::with_sdl(
                r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Unknown argumant named 'other'
        See schema at 29:97:
        {graph: A, name: "meta", arguments: {value: "str", other: 1}}
        "#);
    });
}
