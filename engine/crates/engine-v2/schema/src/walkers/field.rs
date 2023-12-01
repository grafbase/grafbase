use super::SchemaWalker;
use crate::{DataSourceId, FieldId, FieldResolver, FieldSet, InputValueWalker, Resolver, Schema, TypeWalker};

pub type FieldWalker<'a> = SchemaWalker<'a, FieldId>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.field(self.id)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn deprecated_reason(&self) -> Option<&'a str> {
        self.deprecation_reason.map(|id| self.schema[id].as_str())
    }

    pub fn resolvers(&self) -> impl Iterator<Item = FieldResolverWalker<'a>> + 'a {
        let schema = self.schema;
        self.schema[self.id]
            .resolvers
            .iter()
            .map(move |inner| FieldResolverWalker { schema, inner })
    }

    pub fn provides(&self, data_source_id: DataSourceId) -> Option<&'a FieldSet> {
        self.schema[self.id]
            .provides
            .iter()
            .find(|provides| provides.data_source_id == data_source_id)
            .map(|provides| &provides.fields)
    }

    pub fn arguments(&self) -> impl Iterator<Item = InputValueWalker<'a>> + 'a {
        let walker = *self;
        self.schema[self.id].arguments.iter().map(move |id| walker.walk(*id))
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
}

pub struct FieldResolverWalker<'a> {
    schema: &'a Schema,
    inner: &'a FieldResolver,
}

impl<'a> FieldResolverWalker<'a> {
    pub fn resolver(&self) -> &Resolver {
        &self.schema[self.inner.resolver_id]
    }
}

impl<'a> std::ops::Deref for FieldResolverWalker<'a> {
    type Target = FieldResolver;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldWalker")
            .field("id", &usize::from(self.id))
            .field("name", &self.name())
            .field("description", &self.description())
            .field("type", &self.ty())
            .field("resolvers", &self.resolvers)
            .field("is_deprecated", &self.is_deprecated)
            .field("deprecated_reason", &self.deprecated_reason())
            .field(
                "arguments",
                &self
                    .arguments()
                    .map(|arg| (arg.name(), arg.ty().name()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
