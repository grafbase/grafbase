use std::borrow::Cow;

use walker::Walk;

use crate::{
    FieldDefinitionId, GraphqlFederationEntityResolverDefinition, GraphqlRootFieldResolverDefinition, RequiredFieldSet,
    RequiredFieldSetId, RequiredFieldSetRecord, ResolverDefinition, ResolverDefinitionRecord,
    ResolverDefinitionVariant, Subgraph, SubgraphId,
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

    pub fn required_field_set_id(&self) -> Option<RequiredFieldSetId> {
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

    pub fn required_field_set(&self) -> Option<RequiredFieldSet<'a>> {
        self.as_ref().required_field_set_id().walk(self.schema)
    }

    pub fn requires_or_empty(&self) -> &'a RequiredFieldSetRecord {
        self.as_ref()
            .required_field_set_id()
            .map(|id| id.walk(self.schema).as_ref())
            .unwrap_or(RequiredFieldSetRecord::empty())
    }

    pub fn name(&self) -> Cow<'static, str> {
        match self.variant() {
            ResolverDefinitionVariant::Introspection => "Introspection".into(),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => resolver.name().into(),
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => resolver.name().into(),
        }
    }

    pub fn can_provide(&self, field_id: FieldDefinitionId) -> bool {
        field_id.walk(self.schema).is_resolvable_in(self.subgraph_id())
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
