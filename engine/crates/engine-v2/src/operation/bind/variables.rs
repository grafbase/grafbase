use std::collections::{btree_map::Entry, HashSet};

use engine::Positioned;
use schema::{DefinitionId, Schema, Wrapping};

use crate::{
    operation::{Location, Operation, VariableDefinition, VariableInputValues, VariableValue, Variables},
    response::{ErrorCode, GraphqlError},
};

use super::{
    coercion::{coerce_variable, coerce_variable_default_value, InputValueError},
    BindError, BindResult, Binder,
};

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
        GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError).with_locations(locations)
    }
}

pub fn bind_variables(
    schema: &Schema,
    operation: &Operation,
    mut request_variables: engine::Variables,
) -> Result<Variables, Vec<VariableError>> {
    let mut errors = Vec::new();
    let mut variables = Variables {
        input_values: VariableInputValues::default(),
        definition_to_value: vec![VariableValue::Undefined; operation.variable_definitions.len()],
    };

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

impl<'schema, 'p> Binder<'schema, 'p> {
    pub(super) fn bind_variable_definitions(
        &mut self,
        variables: Vec<Positioned<engine_parser::types::VariableDefinition>>,
    ) -> BindResult<Vec<VariableDefinition>> {
        let mut seen_names = HashSet::new();
        let mut bound_variables = Vec::new();

        for Positioned { node, .. } in variables {
            let name = node.name.node.to_string();
            let name_location = node.name.pos.try_into()?;

            if seen_names.contains(&name) {
                return Err(BindError::DuplicateVariable {
                    name,
                    location: name_location,
                });
            }
            seen_names.insert(name.clone());

            let mut ty = self.convert_type(&name, node.var_type.pos.try_into()?, node.var_type.node)?;

            match node.default_value.as_ref().map(|pos| &pos.node) {
                Some(value) if !value.is_null() => {
                    if ty.wrapping.is_list() {
                        ty.wrapping = ty.wrapping.wrapped_by_required_list();
                    } else {
                        ty.wrapping = Wrapping::new(true);
                    }
                }
                _ => (),
            }

            let default_value = node
                .default_value
                .map(|Positioned { pos: _, node: value }| coerce_variable_default_value(self, name_location, ty, value))
                .transpose()?;

            bound_variables.push(VariableDefinition {
                name,
                name_location,
                default_value,
                ty,
                used_by: Vec::new(),
            });
        }

        Ok(bound_variables)
    }

    pub(super) fn validate_all_variables_used(&self) -> BindResult<()> {
        for variable in &self.variable_definitions {
            if variable.used_by.is_empty() {
                return Err(BindError::UnusedVariable {
                    name: variable.name.clone(),
                    operation: self.operation_name.clone(),
                    location: variable.name_location,
                });
            }
        }

        Ok(())
    }

    fn convert_type(
        &self,
        variable_name: &str,
        location: Location,
        ty: engine_parser::types::Type,
    ) -> BindResult<schema::TypeRecord> {
        match ty.base {
            engine_parser::types::BaseType::Named(type_name) => {
                let definition =
                    self.schema
                        .definition_by_name(type_name.as_str())
                        .ok_or_else(|| BindError::UnknownType {
                            name: type_name.to_string(),
                            location,
                        })?;
                if !matches!(
                    definition,
                    DefinitionId::Enum(_) | DefinitionId::Scalar(_) | DefinitionId::InputObject(_)
                ) {
                    return Err(BindError::InvalidVariableType {
                        name: variable_name.to_string(),
                        ty: self.schema.walk(definition).name().to_string(),
                        location,
                    });
                }
                Ok(schema::TypeRecord {
                    definition_id: definition,
                    wrapping: schema::Wrapping::new(!ty.nullable),
                })
            }
            engine_parser::types::BaseType::List(nested) => {
                self.convert_type(variable_name, location, *nested).map(|mut r#type| {
                    if ty.nullable {
                        r#type.wrapping = r#type.wrapping.wrapped_by_nullable_list();
                    } else {
                        r#type.wrapping = r#type.wrapping.wrapped_by_required_list();
                    }
                    r#type
                })
            }
        }
    }
}
