use crate::{FieldDefinitionId, RequiredFieldSet, ResolverDefinition, ResolverDefinitionId, SchemaWalker, SubgraphId};

pub type ResolverDefinitionWalker<'a> = SchemaWalker<'a, ResolverDefinitionId>;

impl<'a> ResolverDefinitionWalker<'a> {
    pub fn name(&self) -> String {
        match self.as_ref() {
            ResolverDefinition::Introspection(_) => "Introspection resolver".to_string(),
            ResolverDefinition::GraphqlRootField(resolver) => self.walk(resolver).name(),
            ResolverDefinition::GraphqlFederationEntity(resolver) => self.walk(resolver).name(),
        }
    }

    pub fn supports_aliases(&self) -> bool {
        match self.as_ref() {
            ResolverDefinition::GraphqlRootField(_)
            | ResolverDefinition::Introspection(_)
            | ResolverDefinition::GraphqlFederationEntity(_) => true,
        }
    }

    pub fn requires(&self) -> &'a RequiredFieldSet {
        match self.as_ref() {
            ResolverDefinition::GraphqlFederationEntity(resolver) => self.walk(resolver).requires(),
            ResolverDefinition::Introspection(_) | ResolverDefinition::GraphqlRootField(_) => &crate::requires::EMPTY,
        }
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        match self.as_ref() {
            ResolverDefinition::Introspection(resolver) => self.walk(resolver).subgraph_id(),
            ResolverDefinition::GraphqlRootField(resolver) => self.walk(resolver).subgraph_id(),
            ResolverDefinition::GraphqlFederationEntity(resolver) => self.walk(resolver).subgraph_id(),
        }
    }

    pub fn can_provide(&self, field_id: FieldDefinitionId) -> bool {
        self.walk(field_id).is_resolvable_in(self.subgraph_id())
    }
}

impl<'a> std::fmt::Debug for ResolverDefinitionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            ResolverDefinition::Introspection(_) => f.debug_struct("Introspection").finish(),
            ResolverDefinition::GraphqlRootField(resolver) => self.walk(resolver).fmt(f),
            ResolverDefinition::GraphqlFederationEntity(resolver) => self.walk(resolver).fmt(f),
        }
    }
}
