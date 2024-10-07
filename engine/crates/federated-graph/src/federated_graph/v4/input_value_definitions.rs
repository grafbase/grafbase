use crate::ViewNested;

use super::{
    ArgumentDefinitionId, Directives, FederatedGraph, FieldId, InputObjectFieldDefinitionId, InputValueDefinitionId,
    StringId, Type, TypeDefinitionId, Value,
};

pub type InputObjectField<'a> = ViewNested<'a, InputObjectFieldDefinitionId, InputObjectFieldDefinitionRecord>;
pub type ArgumentDefinition<'a> = ViewNested<'a, ArgumentDefinitionId, ArgumentDefinitionRecord>;
pub type InputValueDefinition<'a> = ViewNested<'a, InputValueDefinitionId, InputValueDefinitionRecord>;

#[derive(Clone, PartialEq)]
pub struct InputObjectFieldDefinitionRecord {
    pub input_object_id: TypeDefinitionId,
    pub input_value_definition_id: InputValueDefinitionId,
}

#[derive(Clone, PartialEq)]
pub struct ArgumentDefinitionRecord {
    pub field_id: FieldId,
    pub input_value_definition_id: InputValueDefinitionId,
}

#[derive(Clone, PartialEq)]
pub struct InputValueDefinitionRecord {
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
    pub fn iter_field_arguments(&self, field_id: FieldId) -> impl Iterator<Item = ArgumentDefinition<'_>> {
        self.iter_by_sort_key(field_id, &self.argument_definitions, |record| record.field_id)
    }

    pub fn iter_input_object_fields(
        &self,
        input_object_id: TypeDefinitionId,
    ) -> impl Iterator<Item = InputObjectField<'_>> {
        self.iter_by_sort_key(input_object_id, &self.input_object_field_definitions, |record| {
            record.input_object_id
        })
    }

    pub fn push_input_value_definition(&mut self, record: InputValueDefinitionRecord) -> InputValueDefinitionId {
        let id = InputValueDefinitionId::from(self.input_value_definitions.len());
        self.input_value_definitions.push(record);
        id
    }
}
