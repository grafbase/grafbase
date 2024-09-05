use super::SchemaWalker;
use crate::{ObjectDefinition, TypeSystemDirectivesWalker, UnionDefinitionId};

pub type UnionDefinition<'a> = SchemaWalker<'a, UnionDefinitionId>;

impl<'a> UnionDefinition<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }

    pub fn possible_types(self) -> impl ExactSizeIterator<Item = ObjectDefinition<'a>> + 'a {
        self.as_ref()
            .possible_type_ids
            .clone()
            .into_iter()
            .map(move |id| self.walk(id))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directive_ids)
    }
}

impl<'a> std::fmt::Debug for UnionDefinition<'a> {
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
