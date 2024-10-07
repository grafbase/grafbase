use crate::ViewNested;

use super::{Directives, FederatedGraph, FieldId, InputValueDefinitionId, StringId, Type, TypeDefinitionId, Value};

pub type InputValueDefinition<'a> = ViewNested<'a, InputValueDefinitionId, InputValueDefinitionRecord>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Copy)]
pub enum InputValueDefinitionLocation {
    Argument(FieldId),
    InputObject(TypeDefinitionId),
}

impl From<FieldId> for InputValueDefinitionLocation {
    fn from(value: FieldId) -> Self {
        Self::Argument(value)
    }
}

impl From<TypeDefinitionId> for InputValueDefinitionLocation {
    fn from(value: TypeDefinitionId) -> Self {
        Self::InputObject(value)
    }
}

#[derive(Clone, PartialEq)]
pub struct InputValueDefinitionRecord {
    pub location: InputValueDefinitionLocation,
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
    pub default: Option<Value>,
}

pub type InputValueDefinitionSet = Vec<InputValueDefinitionSetItem>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct InputValueDefinitionSetItem {
    pub input_value_definition: InputValueDefinitionId,
    pub subselection: InputValueDefinitionSet,
}

impl FederatedGraph {
    pub fn input_value_definitions_range(
        &self,
        location: InputValueDefinitionLocation,
    ) -> (InputValueDefinitionId, usize) {
        let mut values = self.iter_input_value_definitions(location);
        let Some(start) = values.next() else {
            return (InputValueDefinitionId::from(0), 0);
        };

        (start.id(), values.count() + 1)
    }

    pub fn iter_field_arguments(&self, field_id: FieldId) -> impl Iterator<Item = InputValueDefinition<'_>> {
        self.iter_input_value_definitions(field_id.into())
    }

    pub fn iter_input_value_definitions(
        &self,
        location: InputValueDefinitionLocation,
    ) -> impl Iterator<Item = InputValueDefinition<'_>> {
        self.iter_by_sort_key(location, &self.input_value_definitions, |record| record.location)
    }

    pub fn iter_input_object_fields(
        &self,
        input_object_id: TypeDefinitionId,
    ) -> impl Iterator<Item = InputValueDefinition<'_>> {
        self.iter_input_value_definitions(input_object_id.into())
    }

    pub fn push_input_value_definition(&mut self, record: InputValueDefinitionRecord) -> InputValueDefinitionId {
        let id = InputValueDefinitionId::from(self.input_value_definitions.len());
        self.input_value_definitions.push(record);
        id
    }
}
