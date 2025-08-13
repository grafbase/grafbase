use std::borrow::Cow;

use walker::Walk;

use crate::{
    ExtensionResolverDefinition, FieldResolverExtensionDefinition, FieldSet, GraphqlFederationEntityResolverDefinition,
    GraphqlRootFieldResolverDefinition, LookupResolverDefinition, ResolverDefinition, ResolverDefinitionVariant,
    SelectionSetResolverExtensionDefinition, Subgraph, SubgraphId,
};

impl<'a> ResolverDefinition<'a> {
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id().walk(self.schema)
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        match self.variant() {
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => resolver.subgraph_id.into(),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => resolver.subgraph_id.into(),
            ResolverDefinitionVariant::Introspection(_) => SubgraphId::Introspection,
            ResolverDefinitionVariant::FieldResolverExtension(resolver) => resolver.directive().subgraph_id,
            ResolverDefinitionVariant::Extension(resolver) => resolver.subgraph_id.into(),
            ResolverDefinitionVariant::SelectionSetResolverExtension(resolver) => resolver.subgraph_id.into(),
            ResolverDefinitionVariant::Lookup(resolver) => resolver.resolver().subgraph_id(),
        }
    }

    pub fn required_field_set(&self) -> Option<FieldSet<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => Some(resolver.key_fields()),
            ResolverDefinitionVariant::FieldResolverExtension(resolver) => Some(resolver.directive().requirements()),
            ResolverDefinitionVariant::Lookup(resolver) => Some(resolver.key()),
            _ => None,
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.variant() {
            ResolverDefinitionVariant::Introspection(_) => "Introspection".into(),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::FieldResolverExtension(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::Extension(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::Lookup(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::SelectionSetResolverExtension(resolver) => resolver.name().into(),
        }
    }
}

impl FieldResolverExtensionDefinition<'_> {
    pub fn name(&self) -> String {
        format!("{}#{}", self.directive().name(), self.directive().subgraph().name())
    }
}

impl GraphqlRootFieldResolverDefinition<'_> {
    pub fn name(&self) -> String {
        format!("Root#{}", self.subgraph().name())
    }
}

impl GraphqlFederationEntityResolverDefinition<'_> {
    pub fn name(&self) -> String {
        format!("FedEntity#{}", self.subgraph().name())
    }
}

impl LookupResolverDefinition<'_> {
    pub fn name(&self) -> String {
        format!("Lookup/{}", self.resolver().name())
    }
}

impl ExtensionResolverDefinition<'_> {
    pub fn name(&self) -> String {
        format!(
            "{}/{}#{}",
            self.directive().name(),
            self.schema[self.extension_id],
            self.subgraph().name()
        )
    }
}

impl SelectionSetResolverExtensionDefinition<'_> {
    pub fn name(&self) -> String {
        format!(
            "SelectionSetResolver#{}#{}",
            usize::from(self.extension_id),
            self.subgraph().name()
        )
    }
}
