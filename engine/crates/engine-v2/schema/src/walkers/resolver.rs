use std::borrow::Cow;

use crate::{FieldSet, Names, Resolver, ResolverGroup, ResolverId, SchemaWalker};

pub type ResolverWalker<'a> = SchemaWalker<'a, ResolverId>;

impl<'a> ResolverWalker<'a> {
    pub fn name(&self) -> String {
        match self.get() {
            Resolver::Introspection(_) => "Introspection resolver".to_string(),
            Resolver::FederationRootField(resolver) => self.walk(resolver).name(),
            Resolver::FederationEntity(resolver) => self.walk(resolver).name(),
        }
    }

    pub fn supports_aliases(&self) -> bool {
        match self.get() {
            Resolver::FederationRootField(_) | Resolver::Introspection(_) | Resolver::FederationEntity(_) => true,
        }
    }

    pub fn names(&self) -> &'a dyn Names {
        &()
    }

    pub fn requires(&self) -> Cow<'_, FieldSet> {
        match self.get() {
            Resolver::FederationEntity(resolver) => Cow::Borrowed(&resolver.key.fields),
            _ => Cow::Owned(FieldSet::default()),
        }
    }

    pub fn group(&self) -> Option<ResolverGroup> {
        match self.get() {
            Resolver::Introspection(_) => None,
            Resolver::FederationRootField(resolver) => Some(ResolverGroup::Federation(resolver.subgraph_id)),
            Resolver::FederationEntity(resolver) => Some(ResolverGroup::Federation(resolver.subgraph_id)),
        }
    }
}

impl<'a> std::fmt::Debug for ResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Resolver::Introspection(_) => f.debug_struct("Introspection").finish(),
            Resolver::FederationRootField(resolver) => self.walk(resolver).fmt(f),
            Resolver::FederationEntity(resolver) => self.walk(resolver).fmt(f),
        }
    }
}
