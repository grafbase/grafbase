use std::collections::btree_map::Entry;

use schema::{RawInputValue, Schema};

use crate::{
    request::{Location, OpInputValues, Operation},
    response::GraphqlError,
};

use super::coercion::{const_value::coerce_variable, InputValueError};

#[derive(Debug, thiserror::Error)]
pub enum VariableError {
    #[error("Variable ${name} is missing")]
    MissingVariable { name: String, location: Location },
    #[error("Variable ${name} has an invalid value. {err}")]
    InvalidValue { name: String, err: InputValueError },
}

impl From<VariableError> for GraphqlError {
    fn from(err: VariableError) -> Self {
        let locations = match err {
            VariableError::MissingVariable { location, .. } => vec![location],
            VariableError::InvalidValue { ref err, .. } => vec![err.location()],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            ..Default::default()
        }
    }
}

pub fn bind_variables(
    schema: &Schema,
    operation: &Operation,
    variables: &mut engine_value::Variables,
) -> Result<OpInputValues, Vec<VariableError>> {
    let mut input_values = operation.input_values.clone();
    let mut errors = Vec::new();

    for definition in &operation.variable_definitions {
        match variables.entry(engine_value::Name::new(&definition.name)) {
            Entry::Occupied(mut entry) => {
                let value = std::mem::take(entry.get_mut());
                match coerce_variable(
                    schema,
                    &mut input_values,
                    definition.name_location,
                    definition.r#type,
                    value,
                ) {
                    Ok(input_value) => {
                        input_values[definition.future_input_value_id] = input_value;
                    }
                    Err(err) => errors.push(VariableError::InvalidValue {
                        name: definition.name.clone(),
                        err,
                    }),
                }
            }
            Entry::Vacant(_) => {
                if definition.default_value.is_none() {
                    if definition.r#type.wrapping.is_required() {
                        errors.push(VariableError::MissingVariable {
                            name: definition.name.clone(),
                            location: definition.name_location,
                        });
                    } else {
                        input_values[definition.future_input_value_id] = RawInputValue::Undefined;
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(input_values)
}
