use std::borrow::Cow;

use walker::Walk;

use crate::{
    FieldSet, FieldSetId, GraphqlFederationEntityResolverDefinition, GraphqlRootFieldResolverDefinition,
    ResolverDefinition, ResolverDefinitionRecord, ResolverDefinitionVariant, Subgraph, SubgraphId,
};

impl ResolverDefinitionRecord {
    pub fn subgraph_id(&self) -> SubgraphId {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => {
                SubgraphId::GraphqlEndpoint(resolver.endpoint_id)
            }
            ResolverDefinitionRecord::GraphqlRootField(resolver) => SubgraphId::GraphqlEndpoint(resolver.endpoint_id),
            ResolverDefinitionRecord::Introspection => SubgraphId::Introspection,
        }
    }

    pub fn required_field_set_id(&self) -> Option<FieldSetId> {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => Some(resolver.key_fields_id),
            ResolverDefinitionRecord::GraphqlRootField(_) | ResolverDefinitionRecord::Introspection => None,
        }
    }
}

impl<'a> ResolverDefinition<'a> {
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.as_ref().subgraph_id().walk(self.schema)
    }

    pub fn required_field_set(&self) -> Option<FieldSet<'a>> {
        self.as_ref().required_field_set_id().walk(self.schema)
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.variant() {
            ResolverDefinitionVariant::Introspection => "Introspection".into(),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => resolver.name().into(),
        }
    }
}

impl<'a> GraphqlRootFieldResolverDefinition<'a> {
    pub fn name(&self) -> String {
        format!("Root#{}", self.endpoint().subgraph_name())
    }
}

impl<'a> GraphqlFederationEntityResolverDefinition<'a> {
    pub fn name(&self) -> String {
        format!("FedEntity#{}", self.endpoint().subgraph_name())
    }
}
