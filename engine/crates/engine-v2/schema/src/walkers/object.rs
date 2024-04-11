use super::{FieldDefinitionWalker, SchemaWalker};
use crate::{InterfaceWalker, ObjectId, TypeSystemDirectivesWalker};

pub type ObjectWalker<'a> = SchemaWalker<'a, ObjectId>;

impl<'a> ObjectWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.object(self.schema, self.item)
    }

    pub fn fields(self) -> impl Iterator<Item = FieldDefinitionWalker<'a>> + 'a {
        let fields = self.schema[self.item].fields;
        fields.map(move |field_id| self.walk(field_id))
    }

    pub fn interfaces(self) -> impl ExactSizeIterator<Item = InterfaceWalker<'a>> + 'a {
        self.as_ref()
            .interfaces
            .clone()
            .into_iter()
            .map(move |id| self.walk(id))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for ObjectWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().map(|i| i.name()).collect::<Vec<_>>())
            .finish()
    }
}
