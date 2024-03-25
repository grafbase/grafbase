use crate::{FieldDefinitionId, Names, RequiredFieldSet, Resolver, ResolverId, SchemaWalker, SubgraphId};

pub type ResolverWalker<'a> = SchemaWalker<'a, ResolverId>;

impl<'a> ResolverWalker<'a> {
    pub fn name(&self) -> String {
        match self.as_ref() {
            Resolver::Introspection(_) => "Introspection resolver".to_string(),
            Resolver::GraphqlRootField(resolver) => self.walk(resolver).name(),
            Resolver::GraphqlFederationEntity(resolver) => self.walk(resolver).name(),
        }
    }

    pub fn supports_aliases(&self) -> bool {
        match self.as_ref() {
            Resolver::GraphqlRootField(_) | Resolver::Introspection(_) | Resolver::GraphqlFederationEntity(_) => true,
        }
    }

    pub fn with_own_names(&self) -> Self {
        self.schema.walker_with(self.names()).walk(self.id())
    }

    pub fn names(&self) -> &'a dyn Names {
        &()
    }

    pub fn requires(&self) -> &'a RequiredFieldSet {
        match self.as_ref() {
            Resolver::GraphqlFederationEntity(resolver) => self.walk(resolver).requires(),
            Resolver::Introspection(_) | Resolver::GraphqlRootField(_) => &crate::requires::EMPTY,
        }
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        match self.as_ref() {
            Resolver::Introspection(resolver) => self.walk(resolver).subgraph_id(),
            Resolver::GraphqlRootField(resolver) => self.walk(resolver).subgraph_id(),
            Resolver::GraphqlFederationEntity(resolver) => self.walk(resolver).subgraph_id(),
        }
    }

    pub fn can_provide(&self, field_id: FieldDefinitionId) -> bool {
        self.walk(field_id).is_resolvable_in(self.subgraph_id())
    }
}

impl<'a> std::fmt::Debug for ResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            Resolver::Introspection(_) => f.debug_struct("Introspection").finish(),
            Resolver::GraphqlRootField(resolver) => self.walk(resolver).fmt(f),
            Resolver::GraphqlFederationEntity(resolver) => self.walk(resolver).fmt(f),
        }
    }
}
