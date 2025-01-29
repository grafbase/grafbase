use runtime::{
    error::PartialGraphqlError,
    extension::{Data, ExtensionDirective, ExtensionDirectiveKind, ExtensionId},
    hooks::{Anything, DynHookContext, EdgeDefinition},
};
use serde::Deserialize;

#[derive(Default)]
pub struct TestExtensions {
    extensions: Vec<extension::Id>,
    field_resolvers: Vec<FieldResolver>,
}

struct FieldResolver {
    id: ExtensionId,
    resolver: Box<dyn TestFieldResolvereExtension>,
    directives: Vec<String>,
}

impl TestExtensions {
    pub fn with_field_resolver(
        mut self,
        id: extension::Id,
        directives: &[&str],
        resolver: impl TestFieldResolvereExtension + 'static,
    ) -> Self {
        self.field_resolvers.push(FieldResolver {
            id: self.extensions.len().into(),
            resolver: Box::new(resolver),
            directives: directives.iter().map(|s| s.to_string()).collect(),
        });
        self.extensions.push(id);
        self
    }
}

#[async_trait::async_trait]
pub trait TestFieldResolvereExtension: Send + Sync + 'static {
    async fn resolve<'a>(
        &self,
        subgraph_directives: Vec<ExtensionDirective<'a, serde_json::Value>>,
        context: &DynHookContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError>;
}

impl runtime::extension::ExtensionCatalog for TestExtensions {
    fn find_compatible_extension(&self, id: &extension::Id) -> Option<ExtensionId> {
        self.extensions
            .iter()
            .enumerate()
            .find(|(_, existing)| existing.is_compatible_with(id))
            .map(|(ix, _)| ix.into())
    }

    fn get_directive_kind(&self, id: ExtensionId, name: &str) -> runtime::extension::ExtensionDirectiveKind {
        if self
            .field_resolvers
            .iter()
            .any(|res| res.id == id && res.directives.iter().any(|s| s == name))
        {
            ExtensionDirectiveKind::FieldResolver
        } else {
            ExtensionDirectiveKind::Unknown
        }
    }
}

impl runtime::extension::ExtensionRuntime for TestExtensions {
    type SharedContext = DynHookContext;

    async fn resolve_field<'a>(
        &self,
        id: ExtensionId,
        subgraph_directives: impl IntoIterator<Item = ExtensionDirective<'a, impl Anything<'a>>> + Send,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<runtime::extension::Data, PartialGraphqlError>>, PartialGraphqlError> {
        let Some(FieldResolver { resolver, .. }) = self.field_resolvers.iter().find(|res| res.id == id) else {
            return Err(PartialGraphqlError::internal_hook_error());
        };

        resolver
            .resolve(
                subgraph_directives
                    .into_iter()
                    .map(|ExtensionDirective { name, static_arguments }| ExtensionDirective {
                        name,
                        static_arguments: serde_json::Value::deserialize(static_arguments).unwrap(),
                    })
                    .collect(),
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
