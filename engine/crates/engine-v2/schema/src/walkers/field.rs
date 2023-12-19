use super::{resolver::ResolverWalker, SchemaWalker};
use crate::{CacheConfig, FieldId, FieldResolver, FieldSet, InputValueWalker, TypeWalker};

pub type FieldWalker<'a> = SchemaWalker<'a, FieldId>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.field(self.schema, self.wrapped)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn deprecated_reason(&self) -> Option<&'a str> {
        self.deprecation_reason.map(|id| self.schema[id].as_str())
    }

    pub fn resolvers(&self) -> impl Iterator<Item = FieldResolverWalker<'a>> + 'a {
        let walker = self.walk(());
        self.schema[self.wrapped]
            .resolvers
            .iter()
            .map(move |FieldResolver { resolver_id, requires }| FieldResolverWalker {
                resolver: walker.walk(*resolver_id),
                requires,
            })
    }

    pub fn arguments(&self) -> impl Iterator<Item = InputValueWalker<'a>> + 'a {
        let walker = *self;
        self.schema[self.wrapped]
            .arguments
            .iter()
            .map(move |id| walker.walk(*id))
    }

    pub fn argument_by_name(&self, name: &str) -> Option<InputValueWalker<'a>> {
        self.arguments
            .iter()
            .find(|argument_id| self.schema[self.schema[**argument_id].name] == name)
            .map(|id| self.walk(*id))
    }

    pub fn ty(self) -> TypeWalker<'a> {
        self.walk(self.type_id)
    }

    pub fn cache_config(&self) -> Option<CacheConfig> {
        self.cache_config.map(|cache_config_id| self.schema[cache_config_id])
    }
}

pub struct FieldResolverWalker<'a> {
    pub resolver: ResolverWalker<'a>,
    pub requires: &'a FieldSet,
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("id", &usize::from(self.wrapped))
            .field("name", &self.name())
            .field("type", &self.ty().to_string())
            .field("resolvers", &self.resolvers().map(|fr| fr.resolver).collect::<Vec<_>>())
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
