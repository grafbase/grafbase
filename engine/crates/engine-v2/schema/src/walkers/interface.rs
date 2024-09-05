use super::{FieldDefinition, SchemaWalker};
use crate::{InterfaceDefinitionId, ObjectDefinition, TypeSystemDirectivesWalker};

pub type InterfaceDefinition<'a> = SchemaWalker<'a, InterfaceDefinitionId>;

impl<'a> InterfaceDefinition<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }

    pub fn fields(self) -> impl Iterator<Item = FieldDefinition<'a>> + 'a {
        let fields = self.schema[self.item].field_ids;
        fields.into_iter().map(move |field_id| self.walk(field_id))
    }

    pub fn interfaces(self) -> impl ExactSizeIterator<Item = InterfaceDefinition<'a>> + 'a {
        self.as_ref()
            .interface_ids
            .clone()
            .into_iter()
            .map(move |id| self.walk(id))
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

impl<'a> std::fmt::Debug for InterfaceDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .field(
                "possible_types",
                &self.possible_types().map(|t| t.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
