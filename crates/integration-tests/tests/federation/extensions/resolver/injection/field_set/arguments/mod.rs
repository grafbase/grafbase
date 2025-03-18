mod default_value;
mod enum_;
mod fields;
mod list;
mod non_null;
mod scalar;

use std::sync::Arc;

use engine::GraphqlError;
use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestManifest, json_data};
use runtime::extension::{Data, ExtensionFieldDirective};

#[derive(Default)]
pub struct DoubleEchoExt;

impl TestExtensionBuilder for DoubleEchoExt {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: "echo".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            kind: extension_catalog::Kind::Resolver(extension_catalog::ResolverKind {
                resolver_directives: Some(vec!["echo".to_string(), "echoArgs".to_string()]),
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
    async fn resolve_field(
        &self,
        _headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'_, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        match directive.name {
            "echo" => Ok(inputs
                .into_iter()
                .map(|input| Ok(json_data(&input["fields"])))
                .collect()),
            "echoArgs" => Ok(inputs
                .into_iter()
                .map(|_| Ok(json_data(&directive.arguments["args"])))
                .collect()),
            _ => unreachable!(),
        }
    }
}
