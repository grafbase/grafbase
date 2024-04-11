use schema::FieldDefinitionWalker;

use super::{FieldArgumentsWalker, OperationWalker, SelectionSetWalker};
use crate::{
    operation::{Field, FieldId, Location},
    response::ResponseKey,
};

pub type FieldWalker<'a> = OperationWalker<'a, FieldId>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.as_ref()
            .definition_id()
            .map(|id| self.schema_walker.walk(id).name())
            .unwrap_or("__typename")
    }

    pub fn definition(&self) -> Option<FieldDefinitionWalker<'a>> {
        self.as_ref().definition_id().map(|id| self.schema_walker.walk(id))
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().response_key()
    }

    pub fn response_key_str(&self) -> &'a str {
        self.operation.response_keys.try_resolve(self.response_key()).unwrap()
    }

    pub fn arguments(self) -> FieldArgumentsWalker<'a> {
        self.walk_with(self.as_ref().argument_ids(), ())
    }

    pub fn location(&self) -> Location {
        self.as_ref().location()
    }

    pub fn alias(&self) -> Option<&'a str> {
        Some(self.response_key_str()).filter(|key| key != &self.name())
    }

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }

    pub fn is_extra(&self) -> bool {
        matches!(self.as_ref(), Field::Extra { .. })
    }
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            Field::TypeName { .. } => "__typename".fmt(f),
            Field::Query {
                field_definition_id: field_id,
                ..
            } => {
                let mut fmt = f.debug_struct("Field");
                fmt.field("id", &self.item);
                let name = self.schema_walker.walk(*field_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
            Field::Extra {
                field_definition_id: field_id,
                ..
            } => {
                let mut fmt = f.debug_struct("ExtraField");
                fmt.field("id", &self.item);
                let name = self.schema_walker.walk(*field_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
        }
    }
}
