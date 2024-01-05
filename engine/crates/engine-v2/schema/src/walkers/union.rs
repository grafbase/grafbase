use super::SchemaWalker;
use crate::{ObjectWalker, UnionId};

pub type UnionWalker<'a> = SchemaWalker<'a, UnionId>;

impl<'a> UnionWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.union(self.schema, self.item)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.as_ref().description.map(|id| self.schema[id].as_str())
    }

    pub fn possible_types(&self) -> impl Iterator<Item = ObjectWalker<'a>> + 'a {
        let walker = *self;
        self.as_ref()
            .possible_types
            .clone()
            .into_iter()
            .map(move |id| walker.walk(id))
    }
}

impl<'a> std::fmt::Debug for UnionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Union")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "possible_types",
                &self.possible_types().map(|t| t.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
