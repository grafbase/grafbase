use std::path::Path;

use engine_schema::Subgraph;
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Id};
use runtime::{
    error::PartialGraphqlError,
    extension::{Data, ExtensionDirective},
    hooks::{Anything, DynHookContext, EdgeDefinition},
};
use serde::Deserialize;

#[derive(Default)]
pub struct TestExtensions {
    catalog: ExtensionCatalog,
    field_resolvers: Vec<FieldResolver>,
}

struct FieldResolver {
    id: ExtensionId,
    resolver: Box<dyn TestFieldResolvereExtension>,
}

impl TestExtensions {
    #[track_caller]
    pub fn with_field_resolver(
        mut self,
        path: &Path,
        id: Id,
        directives: &[&str],
        resolver: impl TestFieldResolvereExtension + 'static,
    ) -> Self {
        let manifest = extension_catalog::Manifest {
            id: id.clone(),
            kind: extension_catalog::Kind::FieldResolver(extension_catalog::FieldResolver {
                resolver_directives: directives.iter().map(|s| s.to_string()).collect(),
            }),
            sdk_version: "0.0.0".parse().unwrap(),
            minimum_gateway_version: "0.0.0".parse().unwrap(),
            sdl: None,
        };
        std::fs::write(
            path.join("manifest.json"),
            serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
        )
        .unwrap();
        let id = self.catalog.push(Extension {
            manifest,
            wasm_path: Default::default(),
        });
        self.field_resolvers.push(FieldResolver {
            id,
            resolver: Box::new(resolver),
        });
        self
    }

    pub fn catalog(&self) -> &ExtensionCatalog {
        &self.catalog
    }
}

#[async_trait::async_trait]
pub trait TestFieldResolvereExtension: Send + Sync + 'static {
    async fn resolve<'a>(
        &self,
        context: &DynHookContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError>;
}

impl runtime::extension::ExtensionRuntime for TestExtensions {
    type SharedContext = DynHookContext;

    async fn resolve_field<'a>(
        &self,
        extension_id: ExtensionId,
        _subgraph: Subgraph<'a>,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<runtime::extension::Data, PartialGraphqlError>>, PartialGraphqlError> {
        let Some(FieldResolver { resolver, .. }) = self.field_resolvers.iter().find(|res| res.id == extension_id)
        else {
            return Err(PartialGraphqlError::internal_hook_error());
        };

        resolver
            .resolve(
                context,
                field,
                ExtensionDirective {
                    name: directive.name,
                    static_arguments: serde_json::Value::deserialize(directive.static_arguments).unwrap(),
                },
                inputs
                    .into_iter()
                    .map(serde_json::Value::deserialize)
                    .collect::<Result<_, _>>()
                    .unwrap(),
            )
            .await
            .map(|items| {
                items
                    .into_iter()
                    .map(|res| res.map(|value| Data::JsonBytes(serde_json::to_vec(&value).unwrap())))
                    .collect()
            })
    }
}
