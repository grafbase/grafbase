use schema::Schema;

use super::{
    bind::{bind_variables, VariableError},
    FieldId, Location, Operation, QueryInputValueId, VariableDefinitionId, VariableInputValueId, VariableInputValues,
};

#[derive(Clone)]
pub struct VariableDefinition {
    pub name: String,
    pub name_location: Location,
    pub default_value: Option<QueryInputValueId>,
    /// Keeping track of every field that used this variable.
    /// Used to know whether the variable is used, not much more as of today.
    /// Sorted.
    pub used_by: Vec<FieldId>,
    pub ty: schema::Type,
}

pub struct Variables {
    pub input_values: VariableInputValues,
    pub definition_to_input_value: Vec<Option<VariableInputValueId>>,
}

impl std::ops::Index<VariableDefinitionId> for Variables {
    type Output = Option<VariableInputValueId>;

    fn index(&self, index: VariableDefinitionId) -> &Self::Output {
        &self.definition_to_input_value[usize::from(index)]
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

    pub(super) fn empty_for(operation: &Operation) -> Self {
        Variables {
            input_values: VariableInputValues::default(),
            definition_to_input_value: vec![None; operation.variable_definitions.len()],
        }
    }
}
