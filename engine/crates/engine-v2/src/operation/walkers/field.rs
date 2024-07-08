use schema::{FieldDefinitionWalker, RequiredField};

use super::{FieldArgumentsWalker, OperationWalker, SelectionSetWalker};
use crate::{
    operation::{ExtraField, Field, FieldId, Location, QueryField, SelectionSetType},
    response::ResponseKey,
};

pub type FieldWalker<'a> = OperationWalker<'a, FieldId, ()>;

impl<'a> FieldWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.as_ref()
            .definition_id()
            .map(|id| self.schema_walker.walk(id).name())
            .unwrap_or("__typename")
    }

    pub fn type_condition(&self) -> SelectionSetType {
        match self.as_ref() {
            Field::TypeName(f) => f.type_condition,
            Field::Query(f) => self
                .schema_walker
                .walk(f.field_definition_id)
                .parent_entity()
                .id()
                .into(),
            Field::Extra(f) => self
                .schema_walker
                .walk(f.field_definition_id)
                .parent_entity()
                .id()
                .into(),
        }
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

    pub fn selection_set(&self) -> Option<SelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }

    pub fn parent_selection_set(&self) -> SelectionSetWalker<'a> {
        self.walk_with(self.as_ref().parent_selection_set_id(), ())
    }
}

impl PartialEq<RequiredField> for FieldWalker<'_> {
    fn eq(&self, other: &RequiredField) -> bool {
        if self.definition().expect("Cannot required __typename").id() != other.definition_id {
            return false;
        }

        let input_values = self
            .arguments()
            .into_iter()
            .map(|arg| (arg.as_ref().input_value_definition_id, arg.value()));

        if input_values.len() < other.arguments.len() {
            return false;
        }

        for (definition_id, input_value) in input_values {
            if let Ok(i) = other.arguments.binary_search_by(|probe| probe.0.cmp(&definition_id)) {
                if !input_value
                    .map(|v| v.eq(&self.schema_walker[other.arguments[i].1]))
                    .unwrap_or_default()
                {
                    return false;
                }
            } else if input_value.is_some() {
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
            Field::Query(QueryField {
                field_definition_id: field_id,
                ..
            }) => {
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
            Field::Extra(ExtraField {
                field_definition_id: field_id,
                ..
            }) => {
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
