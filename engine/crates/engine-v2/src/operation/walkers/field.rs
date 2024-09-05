use schema::{FieldDefinition, RequiredField};

use super::{OperationWalker, SelectionSetWalker};
use crate::{
    operation::{ExtraField, Field, FieldId, Location, QueryField},
    response::{ResponseEdge, ResponseKey},
};

pub type FieldWalker<'a> = OperationWalker<'a, FieldId, ()>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.as_ref()
            .definition_id()
            .map(|id| self.schema_walker.walk(id).name())
            .unwrap_or("__typename")
    }

    pub fn definition(&self) -> Option<FieldDefinition<'a>> {
        self.as_ref().definition_id().map(|id| self.schema_walker.walk(id))
    }

    pub fn response_edge(&self) -> ResponseEdge {
        self.as_ref().response_edge()
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().response_key()
    }

    pub fn response_key_str(&self) -> &'a str {
        self.operation.response_keys.try_resolve(self.response_key()).unwrap()
    }

    pub fn location(&self) -> Location {
        self.as_ref().location()
    }

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }
}

impl PartialEq<RequiredField> for FieldWalker<'_> {
    fn eq(&self, required: &RequiredField) -> bool {
        if self.definition().expect("Cannot required __typename").id() != required.definition_id {
            return false;
        }

        let arguments = &self.operation[self.as_ref().argument_ids()];

        if arguments.len() != required.arguments.len() {
            return false;
        }

        for argument in arguments {
            let definition_id = argument.input_value_definition_id;
            let input_value = self.walk(&self.operation.query_input_values[argument.input_value_id]);
            if let Ok(i) = required.arguments.binary_search_by(|probe| probe.0.cmp(&definition_id)) {
                if !input_value.eq(&self.schema_walker[required.arguments[i].1]) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl<'a> std::fmt::Debug for FieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            Field::TypeName { .. } => "__typename".fmt(f),
            Field::Query(QueryField { definition_id, .. }) => {
                let mut fmt = f.debug_struct("Field");
                fmt.field("id", &self.item);
                let name = self.schema_walker.walk(*definition_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
            Field::Extra(ExtraField { definition_id, .. }) => {
                let mut fmt = f.debug_struct("ExtraField");
                fmt.field("id", &self.item);
                let name = self.schema_walker.walk(*definition_id).name();
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
