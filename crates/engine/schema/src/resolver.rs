use std::borrow::Cow;

use walker::Walk;

use crate::{
    FieldResolverExtensionDefinition, FieldSet, FieldSetId, GraphqlFederationEntityResolverDefinition,
    GraphqlRootFieldResolverDefinition, ResolverDefinition, ResolverDefinitionRecord, ResolverDefinitionVariant,
    Subgraph, SubgraphId,
};

impl ResolverDefinitionRecord {
    pub fn required_field_set_id(&self) -> Option<FieldSetId> {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => Some(resolver.key_fields_id),
            ResolverDefinitionRecord::GraphqlRootField(_)
            | ResolverDefinitionRecord::FieldResolverExtension(_)
            | ResolverDefinitionRecord::Introspection => None,
        }
    }
}

impl<'a> ResolverDefinition<'a> {
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id().walk(self.schema)
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        match self.variant() {
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => {
                SubgraphId::GraphqlEndpoint(resolver.endpoint_id)
            }
            ResolverDefinitionVariant::GraphqlRootField(resolver) => SubgraphId::GraphqlEndpoint(resolver.endpoint_id),
            ResolverDefinitionVariant::Introspection => SubgraphId::Introspection,
            ResolverDefinitionVariant::FieldResolverExtension(resolver) => resolver.directive().subgraph_id,
        }
    }

    pub fn required_field_set(&self) -> Option<FieldSet<'a>> {
        self.as_ref().required_field_set_id().walk(self.schema)
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.variant() {
            ResolverDefinitionVariant::Introspection => "Introspection".into(),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::FieldResolverExtension(resolver) => resolver.name().into(),
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
        format!("Root#{}", self.endpoint().subgraph_name())
    }
}

impl GraphqlFederationEntityResolverDefinition<'_> {
    pub fn name(&self) -> String {
        format!("FedEntity#{}", self.endpoint().subgraph_name())
    }
}
