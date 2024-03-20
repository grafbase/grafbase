use schema::{FieldDefinitionId, FieldDefinitionWalker};

use super::{FieldArgumentWalker, OperationWalker, SelectionSetWalker};
use crate::{
    operation::{Field, FieldId, Location, QueryInputValueWalker},
    response::{ResponseEdge, ResponseKey},
};

pub type AnyFieldWalker<'a> = OperationWalker<'a, FieldId>;
pub type FieldWalker<'a> = OperationWalker<'a, FieldId, FieldDefinitionId>;

impl<'a> OperationWalker<'a, FieldId> {
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

    pub fn name_location(&self) -> Option<Location> {
        self.as_ref().name_location()
    }

    pub fn alias(&self) -> Option<&'a str> {
        Some(self.response_key_str()).filter(|key| key != &self.name())
    }

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }
}

impl<'a> std::fmt::Debug for AnyFieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            Field::TypeName { .. } => "__typename".fmt(f),
            Field::SchemaField {
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

impl<'a> FieldWalker<'a> {
    pub fn response_edge(&self) -> ResponseEdge {
        self.as_ref().response_edge()
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().response_key()
    }

    pub fn response_key_str(&self) -> &'a str {
        self.operation.response_keys.try_resolve(self.response_key()).unwrap()
    }

    pub fn arguments(self) -> impl ExactSizeIterator<Item = FieldArgumentWalker<'a>> + 'a {
        self.as_ref()
            .argument_ids()
            .map(move |id| self.walk_with(id, self.operation[id].input_value_definition_id))
    }

    pub fn get_arg_value(&self, name: &str) -> QueryInputValueWalker<'a> {
        self.arguments()
            .find_map(|arg| if arg.name() == name { Some(arg.value()) } else { None })
            .expect("Provided argument must exist in the schema.")
    }

    #[track_caller]
    pub fn get_arg_value_as<T: serde::Deserialize<'a>>(&self, name: &str) -> T {
        T::deserialize(self.get_arg_value(name)).expect("Invalid argument type.")
    }
}

impl<'a> std::ops::Deref for FieldWalker<'a> {
    type Target = FieldDefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}
