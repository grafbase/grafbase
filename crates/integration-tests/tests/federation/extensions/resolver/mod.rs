mod backwards_compatibility;
mod errors;
mod injection;
mod validation;

use std::sync::Arc;

use engine::GraphqlError;
use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestManifest};
use runtime::extension::{Data, ExtensionFieldDirective};

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

impl TestExtensionBuilder for StaticResolverExt {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: "static".to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Resolver(extension_catalog::ResolverType {
                resolver_directives: None,
            }),
            sdl: Some(r#"directive @resolve on FIELD_DEFINITION"#),
        }
    }

    fn build(&self, _: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl TestExtension for StaticResolverExt {
    async fn resolve_field(
        &self,
        _: http::HeaderMap,
        _: ExtensionFieldDirective<'_, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Ok(inputs.into_iter().map(|_| Ok(self.data.clone())).collect())
    }
}
