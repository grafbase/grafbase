use std::borrow::Cow;

use readable::Readable;

use crate::{
    FieldDefinitionId, GraphqlFederationEntityResolverDefinition, GraphqlRootFieldResolverDefinition,
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

    pub fn requires(&self) -> Option<RequiredFieldSetId> {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => Some(resolver.key_fields_id),
            ResolverDefinitionRecord::GraphqlRootField(_) | ResolverDefinitionRecord::Introspection => None,
        }
    }
}

impl<'a> ResolverDefinition<'a> {
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.as_ref().subgraph_id().read(self.schema)
    }

    pub fn requires(&self) -> &'a RequiredFieldSetRecord {
        self.as_ref()
            .requires()
            .map(|id| id.read(self.schema).as_ref())
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
        field_id.read(self.schema).is_resolvable_in(self.subgraph_id())
    }
}

impl<'a> GraphqlRootFieldResolverDefinition<'a> {
    pub fn name(&self) -> String {
        format!(
            "Graphql root field resolver for subgraph '{}'",
            self.endpoint().subgraph_name()
        )
    }
}

impl<'a> GraphqlFederationEntityResolverDefinition<'a> {
    pub fn name(&self) -> String {
        format!(
            "Graphql federation entity resolver for subgraph '{}'",
            self.endpoint().subgraph_name()
        )
    }
}
