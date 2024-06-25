use std::collections::btree_map::Entry;

use schema::Schema;

use crate::{
    operation::{Location, Operation, VariableValue, Variables},
    response::GraphqlError,
};

use super::coercion::{coerce_variable, InputValueError};

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
    mut request_variables: engine::Variables,
) -> Result<Variables, Vec<VariableError>> {
    let mut errors = Vec::new();
    let mut variables = Variables::new_for(operation);

    for (variable_id, definition) in operation.variable_definitions.iter().enumerate() {
        match request_variables.entry(engine_value::Name::new(&definition.name)) {
            Entry::Occupied(mut entry) => {
                let value = std::mem::take(entry.get_mut());
                match coerce_variable(schema, &mut variables.input_values, definition, value) {
                    Ok(id) => variables.definition_to_value[variable_id] = VariableValue::InputValue(id),
                    Err(err) => {
                        errors.push(VariableError::InvalidValue {
                            name: definition.name.clone(),
                            err,
                        });
                    }
                }
            }
            Entry::Vacant(_) => {
                if definition.default_value.is_none() && definition.ty.wrapping.is_required() {
                    errors.push(VariableError::MissingVariable {
                        name: definition.name.clone(),
                        location: definition.name_location,
                    });
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(variables)
}
