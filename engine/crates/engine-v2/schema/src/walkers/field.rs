use std::borrow::Cow;

use super::{resolver::ResolverWalker, SchemaWalker};
use crate::{CacheConfig, FieldId, FieldProvides, FieldResolver, FieldSet, InputValueDefinitionWalker, TypeWalker};

pub type FieldWalker<'a> = SchemaWalker<'a, FieldId>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.field(self.schema, self.item)
    }

    pub fn resolvers(&self) -> impl ExactSizeIterator<Item = FieldResolverWalker<'a>> + 'a {
        let walker = self.walk(());
        self.schema[self.item].resolvers.iter().map(
            move |FieldResolver {
                      resolver_id,
                      field_requires,
                  }| FieldResolverWalker {
                resolver: walker.walk(*resolver_id),
                field_requires,
            },
        )
    }

    pub fn provides_for(&self, resolver: &ResolverWalker<'_>) -> Option<Cow<'a, FieldSet>> {
        let resolver_group = resolver.group();
        self.as_ref()
            .provides
            .iter()
            .filter_map(|provide| match provide {
                FieldProvides::IfResolverGroup { group, field_set } if Some(group) == resolver_group.as_ref() => {
                    Some(Cow::Borrowed(field_set))
                }
                _ => None,
            })
            .reduce(|a, b| Cow::Owned(FieldSet::merge(&a, &b)))
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinitionWalker<'a>> + 'a {
        let walker = *self;
        self.schema[self.item].arguments.iter().map(move |id| walker.walk(*id))
    }

    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinitionWalker<'a>> {
        self.as_ref()
            .arguments
            .iter()
            .find(|argument_id| self.schema[self.schema[**argument_id].name] == name)
            .map(|id| self.walk(*id))
    }

    pub fn ty(self) -> TypeWalker<'a> {
        self.walk(self.as_ref().type_id)
    }

    pub fn cache_config(&self) -> Option<CacheConfig> {
        self.as_ref()
            .cache_config
            .map(|cache_config_id| self.schema[cache_config_id])
    }
}

pub struct FieldResolverWalker<'a> {
    pub resolver: ResolverWalker<'a>,
    pub field_requires: &'a FieldSet,
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("type", &self.ty().to_string())
            .field("resolvers", &self.resolvers().collect::<Vec<_>>())
            .field(
                "arguments",
                &self
                    .arguments()
                    .map(|arg| (arg.name(), arg.ty().to_string()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for FieldResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldResolver")
            .field("resolver", &self.resolver)
            .field("requires", &self.resolver.walk(self.field_requires))
            .finish()
    }
}
