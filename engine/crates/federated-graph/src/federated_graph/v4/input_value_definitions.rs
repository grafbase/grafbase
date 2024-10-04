use crate::ViewNested;

use super::{
    ArgumentDefinitionId, Directives, FederatedGraph, FieldId, InputObjectFieldDefinitionId, StringId, Type,
    TypeDefinitionId, Value,
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
        let start = self
            .input_object_field_definitions
            .partition_point(|record| record.input_object_id < input_object_id);
        self.input_object_field_definitions[start..]
            .iter()
            .take_while(|record| record.input_object_id == input_object_id)
            .enumerate()
            .map(|(idx, record)| ViewNested {
                graph: self,
                view: crate::View {
                    id: InputObjectFieldDefinitionId::from(start + idx),
                    record,
                },
            })
    }
}
