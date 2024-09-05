use super::{FieldDefinition, SchemaWalker};
use crate::{InterfaceDefinition, ObjectDefinitionId, TypeSystemDirectivesWalker};

pub type ObjectDefinition<'a> = SchemaWalker<'a, ObjectDefinitionId>;

impl<'a> ObjectDefinition<'a> {
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

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directive_ids)
    }
}

impl<'a> std::fmt::Debug for ObjectDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .finish()
    }
}
