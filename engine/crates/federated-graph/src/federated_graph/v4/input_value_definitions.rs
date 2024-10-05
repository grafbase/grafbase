use crate::ViewNested;

use super::{
    ArgumentDefinitionId, Directives, FederatedGraph, FieldId, InputObjectFieldDefinitionId, StringId, Type,
    TypeDefinitionId, Value,
};

pub type InputObjectField<'a> = ViewNested<'a, InputObjectFieldDefinitionId, InputObjectFieldDefinitionRecord>;
pub type ArgumentDefinition<'a> = ViewNested<'a, ArgumentDefinitionId, ArgumentDefinitionRecord>;

#[derive(Clone, PartialEq)]
pub struct InputObjectFieldDefinitionRecord {
    pub input_object_id: TypeDefinitionId,
    pub input_value_definition: InputValueDefinition,
}

#[derive(Clone, PartialEq)]
pub struct ArgumentDefinitionRecord {
    pub field_id: FieldId,
    pub input_value_definition: InputValueDefinition,
}

#[derive(Clone, PartialEq)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
    pub default: Option<Value>,
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
}
