mod backwards_compatibility;
mod errors;
mod injection;
mod validation;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::federation::{
    AnyExtension, FieldResolverTestExtension, FieldResolverTestExtensionBuilder, TestManifest,
};
use runtime::extension::Data;

#[derive(Clone)]
pub struct StaticResolverExt {
    data: Data,
}

impl StaticResolverExt {
    pub fn json(bytes: Vec<u8>) -> Self {
        Self {
            data: Data::JsonBytes(bytes),
        }
    }
}

impl AnyExtension for StaticResolverExt {
    fn register(self, state: &mut integration_tests::federation::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "static".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                resolver_directives: None,
            }),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        });
        state.test.field_resolver_builders.insert(id, Arc::new(self));
    }
}

impl FieldResolverTestExtensionBuilder for StaticResolverExt {
    fn build(&self, _: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn FieldResolverTestExtension> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl FieldResolverTestExtension for StaticResolverExt {
    async fn resolve_field(
        &self,
        _directive: ExtensionDirective<'_>,
        _field_definition: FieldDefinition<'_>,
        _prepared_data: &[u8],
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: serde_json::Value,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Ok(inputs.into_iter().map(|_| Ok(self.data.clone())).collect())
    }
}
