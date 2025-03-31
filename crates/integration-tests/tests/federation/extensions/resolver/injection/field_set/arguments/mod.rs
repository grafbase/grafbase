mod default_value;
mod enum_;
mod fields;
mod list;
mod non_null;
mod scalar;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::federation::{
    AnyExtension, ExtensionsBuilder, FieldResolverTestExtension, TestManifest, json_data,
};
use runtime::extension::Data;

#[derive(Default)]
pub struct DoubleEchoExt;

impl AnyExtension for DoubleEchoExt {
    fn register(self, state: &mut ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "echo".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                resolver_directives: Some(vec!["echo".to_string(), "echoArgs".to_string()]),
            }),
            sdl: Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet", "InputValueSet"])

                directive @echoArgs(args: InputValueSet! = "*") on FIELD_DEFINITION
                directive @echo(fields: FieldSet!) on FIELD_DEFINITION
            "#,
            ),
        });

        state.test.field_resolver_builders.insert(
            id,
            Arc::new(|| -> Arc<dyn FieldResolverTestExtension> { Arc::new(DoubleEchoExt) }),
        );
    }
}

#[async_trait::async_trait]
impl FieldResolverTestExtension for DoubleEchoExt {
    async fn resolve_field(
        &self,
        directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        match directive.name() {
            "echo" => Ok(inputs
                .into_iter()
                .map(|input| Ok(json_data(&input["fields"])))
                .collect()),
            "echoArgs" => Ok(inputs
                .into_iter()
                .map(|_| Ok(json_data(&directive_arguments["args"])))
                .collect()),
            _ => unreachable!(),
        }
    }
}
