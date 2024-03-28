use super::SchemaWalker;
use crate::{ObjectWalker, TypeSystemDirectivesWalker, UnionId};

pub type UnionWalker<'a> = SchemaWalker<'a, UnionId>;

impl<'a> UnionWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.union(self.schema, self.item)
    }

    pub fn possible_types(self) -> impl ExactSizeIterator<Item = ObjectWalker<'a>> + 'a {
        self.as_ref()
            .possible_types
            .clone()
            .into_iter()
            .map(move |id| self.walk(id))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for UnionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Union")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field(
                "possible_types",
                &self.possible_types().map(|t| t.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
