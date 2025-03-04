mod default_value;
mod enum_;
mod fields;
mod list;
mod non_null;
mod scalar;

use std::sync::Arc;

use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestExtensionConfig};
use runtime::{error::PartialGraphqlError, extension::ExtensionFieldDirective};

#[derive(Default)]
pub struct DoubleEchoExt;

impl TestExtensionBuilder for DoubleEchoExt {
    fn id(&self) -> Id {
        Id {
            name: "echo".to_string(),
            version: "1.0.0".parse().unwrap(),
        }
    }

    fn config(&self) -> TestExtensionConfig {
        TestExtensionConfig {
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: vec!["echo".to_string(), "echoArgs".to_string()],
            }),
            sdl: Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet", "InputValueSet"])

                directive @echoArgs(args: InputValueSet! = "*") on FIELD_DEFINITION
                directive @echo(fields: FieldSet!) on FIELD_DEFINITION
            "#,
            ),
        }
    }

    fn build(&self, _: Vec<(&str, serde_json::Value)>) -> Arc<dyn TestExtension> {
        Arc::new(DoubleEchoInstance)
    }
}

struct DoubleEchoInstance;

#[async_trait::async_trait]
impl TestExtension for DoubleEchoInstance {
    async fn resolve<'a>(
        &self,
        _headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        match directive.name {
            "echo" => Ok(inputs.into_iter().map(|input| Ok(input["fields"].clone())).collect()),
            "echoArgs" => Ok(inputs
                .into_iter()
                .map(|_| Ok(directive.arguments["args"].clone()))
                .collect()),
            _ => unreachable!(),
        }
    }
}
