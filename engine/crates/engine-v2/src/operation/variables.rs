use schema::Schema;

use super::{
    bind::{bind_variables, VariableError},
    Location, Operation, QueryInputValueId, VariableDefinitionId, VariableInputValueId, VariableInputValues,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableDefinition {
    pub name: String,
    pub name_location: Location,
    pub default_value: Option<QueryInputValueId>,
    pub in_use: bool,
    pub ty: schema::TypeRecord,
}

pub struct Variables {
    pub input_values: VariableInputValues,
    pub definition_to_value: Vec<VariableValue>,
}

#[derive(Clone)]
pub enum VariableValue {
    Undefined,
    InputValue(VariableInputValueId),
}

impl std::ops::Index<VariableDefinitionId> for Variables {
    type Output = VariableValue;

    fn index(&self, index: VariableDefinitionId) -> &Self::Output {
        &self.definition_to_value[usize::from(index)]
    }
}

impl<T> std::ops::Index<T> for Variables
where
    VariableInputValues: std::ops::Index<T>,
{
    type Output = <VariableInputValues as std::ops::Index<T>>::Output;

    fn index(&self, index: T) -> &Self::Output {
        &self.input_values[index]
    }
}

impl Variables {
    pub(crate) fn build(
        schema: &Schema,
        operation: &Operation,
        request_variables: engine::Variables,
    ) -> Result<Self, Vec<VariableError>> {
        bind_variables(schema, operation, request_variables)
    }
}
