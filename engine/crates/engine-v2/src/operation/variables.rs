use schema::Schema;

use super::{
    bind::{bind_variables, VariableError},
    BoundVariableDefinitionId, Location, Operation, QueryInputValueId, VariableInputValueId, VariableInputValues,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundVariableDefinition {
    pub name: String,
    pub name_location: Location,
    pub default_value_id: Option<QueryInputValueId>,
    pub ty_record: schema::TypeRecord,
}

pub(crate) struct Variables {
    pub input_values: VariableInputValues,
    pub definition_to_value: Vec<VariableValueRecord>,
}

#[derive(Clone)]
pub(crate) enum VariableValueRecord {
    Undefined,
    Provided(VariableInputValueId),
    DefaultValue(QueryInputValueId),
}

impl std::ops::Index<BoundVariableDefinitionId> for Variables {
    type Output = VariableValueRecord;

    fn index(&self, index: BoundVariableDefinitionId) -> &Self::Output {
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
        request_variables: crate::request::Variables,
    ) -> Result<Self, Vec<VariableError>> {
        bind_variables(schema, operation, request_variables)
    }
}
