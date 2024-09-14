use std::collections::{btree_map::Entry, HashSet};

use engine::Positioned;
use schema::{DefinitionId, Schema};

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
    /// Binds the variable definitions provided in a list of `VariableDefinition` to their corresponding
    /// types and default values. While binding, it also checks for duplicate variable names and validates the variable
    /// types against the schema. If a duplicate name is encountered, an error is returned. The function returns a
    /// vector of successfully bound `VariableDefinition` objects.
    ///
    /// # Parameters
    ///
    /// - `variables`: A vector of `Positioned<VariableDefinition>` that represents the variable definitions to be bound.
    ///
    /// # Returns
    ///
    /// Returns a `BindResult` containing a vector of bound `VariableDefinition` objects on success, or a `BindError`
    /// on failure.
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

            let ty = self.convert_type(&name, node.var_type.pos.try_into()?, node.var_type.node)?;
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

    /// Validates that all variable definitions have been used.
    ///
    /// This function checks each variable definition in the binder. If any variable is found to be unused,
    /// an error is returned indicating the variable's name, the operation in which it was defined,
    /// and its location in the source.
    ///
    /// # Returns
    ///
    /// Returns a `BindResult` which is `Ok(())` if all variables have been used,
    /// or a `BindError` if there are unused variables.
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

    /// Converts the provided variable type into a corresponding `schema::TypeRecord`.
    ///
    /// This function evaluates the base type of the variable and checks if it is a valid
    /// named type (such as an enum, scalar, or input object). If the type is a list, it
    /// recursively converts the nested type. It also considers whether the type is nullable
    /// or not and adjusts the wrapping appropriately.
    ///
    /// # Parameters
    ///
    /// - `variable_name`: The name of the variable being converted.
    /// - `location`: The location in the source where the type is defined.
    /// - `ty`: The type definition that needs to be converted.
    ///
    /// # Returns
    ///
    /// Returns a `BindResult<schema::TypeRecord>` containing the converted type record if
    /// successful, or a `BindError` if the type is unknown or invalid.
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
