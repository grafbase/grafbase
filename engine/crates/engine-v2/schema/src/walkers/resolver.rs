use crate::{
    FieldDefinitionId, RequiredFieldSet, ResolverDefinitionId, ResolverDefinitionRecord, SchemaWalker, SubgraphId,
};

pub type ResolverDefinition<'a> = SchemaWalker<'a, ResolverDefinitionId>;

impl<'a> ResolverDefinition<'a> {
    pub fn name(&self) -> String {
        match self.as_ref() {
            ResolverDefinitionRecord::Introspection(_) => "Introspection resolver".to_string(),
            ResolverDefinitionRecord::GraphqlRootField(resolver) => self.walk(resolver).name(),
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => self.walk(resolver).name(),
        }
    }

    pub fn supports_aliases(&self) -> bool {
        match self.as_ref() {
            ResolverDefinitionRecord::GraphqlRootField(_)
            | ResolverDefinitionRecord::Introspection(_)
            | ResolverDefinitionRecord::GraphqlFederationEntity(_) => true,
        }
    }

    pub fn requires(&self) -> &'a RequiredFieldSet {
        match self.as_ref() {
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => self.walk(resolver).requires(),
            ResolverDefinitionRecord::Introspection(_) | ResolverDefinitionRecord::GraphqlRootField(_) => {
                &crate::requires::EMPTY
            }
        }
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        match self.as_ref() {
            ResolverDefinitionRecord::Introspection(resolver) => self.walk(resolver).subgraph_id(),
            ResolverDefinitionRecord::GraphqlRootField(resolver) => self.walk(resolver).subgraph_id(),
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => self.walk(resolver).subgraph_id(),
        }
    }

    pub fn can_provide(&self, field_id: FieldDefinitionId) -> bool {
        self.walk(field_id).is_resolvable_in(self.subgraph_id())
    }
}

impl<'a> std::fmt::Debug for ResolverDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            ResolverDefinitionRecord::Introspection(_) => f.debug_struct("Introspection").finish(),
            ResolverDefinitionRecord::GraphqlRootField(resolver) => self.walk(resolver).fmt(f),
            ResolverDefinitionRecord::GraphqlFederationEntity(resolver) => self.walk(resolver).fmt(f),
        }
    }
}
