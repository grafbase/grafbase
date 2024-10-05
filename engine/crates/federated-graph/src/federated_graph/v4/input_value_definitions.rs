use crate::ViewNested;

use super::{
    Directives, FederatedGraph, FieldId, InputObjectFieldDefinitionId, StringId, Type, TypeDefinitionId, Value,
};

type InputObjectField<'a> = ViewNested<'a, InputObjectFieldDefinitionId, InputObjectFieldDefinitionRecord>;

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
    pub fn iter_input_object_fields(
        &self,
        input_object_id: TypeDefinitionId,
    ) -> impl Iterator<Item = InputObjectField<'_>> {
        self.iter_by_sort_key(input_object_id, &self.input_object_field_definitions, |record| {
            record.input_object_id
        })
    }
}
