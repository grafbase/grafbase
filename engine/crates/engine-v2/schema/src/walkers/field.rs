use super::SchemaWalker;
use crate::{FieldId, InputValueWalker, TypeWalker};

pub type FieldWalker<'a> = SchemaWalker<'a, FieldId>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.field(self.id)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn deprecated_reason(&self) -> Option<&'a str> {
        self.deprecated_reason.map(|id| self.schema[id].as_str())
    }

    pub fn arguments<'s>(&'s self) -> impl Iterator<Item = InputValueWalker<'s>> + 's
    where
        'a: 's,
    {
        self.arguments.iter().map(|id| self.walk(*id))
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

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<FieldWalker<'_>>())
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
