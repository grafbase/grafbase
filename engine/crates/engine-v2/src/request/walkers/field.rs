use schema::FieldWalker;

use super::{BoundSelectionSetWalker, OperationWalker};
use crate::{
    request::{BoundField, BoundFieldId, Location},
    response::ResponseKey,
};

pub type BoundFieldWalker<'a> = OperationWalker<'a, BoundFieldId>;

impl<'a> OperationWalker<'a, BoundFieldId> {
    pub fn schema_field(&self) -> Option<FieldWalker<'a>> {
        match self.as_ref() {
            BoundField::Field { field_id, .. } | BoundField::Extra { field_id, .. } => {
                Some(self.schema_walker.walk(*field_id))
            }
            BoundField::TypeName { .. } => None,
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().response_key()
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.response_key()]
    }

    pub fn name_location(&self) -> Option<Location> {
        self.as_ref().name_location()
    }

    pub fn alias(&self) -> Option<&'a str> {
        Some(self.response_key_str()).filter(|&key| match self.as_ref() {
            BoundField::TypeName { .. } => key != "__typename",
            BoundField::Field { field_id, .. } => key != self.schema_walker.walk(*field_id).name(),
            _ => unreachable!(),
        })
    }

    pub fn selection_set(&self) -> Option<BoundSelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            BoundField::TypeName { .. } => "__typename".fmt(f),
            BoundField::Field { field_id, .. } => {
                let mut fmt = f.debug_struct("BoundField");
                fmt.field("id", &self.item);
                let name = self.schema_walker.walk(*field_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
            BoundField::Extra { field_id, .. } => {
                let mut fmt = f.debug_struct("ExtraBoundField");
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
