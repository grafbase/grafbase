mod backwards_compatibility;
mod errors;
mod injection;
mod validation;

use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use extension_catalog::Id;
use integration_tests::gateway::{AnyExtension, FieldResolverTestExtension, TestManifest};
use runtime::extension::Data;

#[derive(Clone)]
pub struct StaticFieldResolverExt {
    data: Data,
}

impl StaticFieldResolverExt {
    pub fn json(bytes: Vec<u8>) -> Self {
        Self {
            data: Data::JsonBytes(bytes),
        }
    }
}

impl AnyExtension for StaticFieldResolverExt {
    fn register(self, state: &mut integration_tests::gateway::ExtensionsBuilder) {
        let id = state.push_test_extension(TestManifest {
            id: Id {
                name: "static".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::FieldResolver(extension_catalog::FieldResolverType {
                resolver_directives: None,
            }),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        });
        state.test.field_resolver_builders.insert(
            id,
            Arc::new(move || -> Arc<dyn FieldResolverTestExtension> { Arc::new(self.clone()) }),
        );
    }
}

#[async_trait::async_trait]
impl FieldResolverTestExtension for StaticFieldResolverExt {
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
