use std::collections::HashSet;

use schema::{Definition, Schema, Wrapping};
use walker::Walk;

use crate::{
    request::RawVariables, Error, Location, Operation, OperationContext, VariableDefinitionRecord, VariableInputValues,
    VariableValueRecord, Variables,
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

impl From<VariableError> for Error {
    fn from(val: VariableError) -> Self {
        let location = match val {
            VariableError::MissingVariable { location, .. } => location,
            VariableError::InvalidValue { ref err, .. } => err.location(),
        };
        Error::validation(val.to_string()).with_location(location)
    }
}

pub fn bind_variables(
    schema: &Schema,
    operation: &Operation,
    mut request_variables: RawVariables,
) -> Result<Variables, Vec<VariableError>> {
    let ctx = OperationContext { schema, operation };
    let mut errors = Vec::new();
    let mut variables = Variables {
        input_values: VariableInputValues::default(),
        definition_to_value: vec![VariableValueRecord::Undefined; operation.variable_definitions.len()],
    };

    for definition in ctx.variable_definitions() {
        match request_variables.remove(&definition.name) {
            Some(value) => match coerce_variable(schema, &mut variables.input_values, definition, value) {
                Ok(id) => variables[definition.id] = VariableValueRecord::Provided(id),
                Err(err) => {
                    errors.push(VariableError::InvalidValue {
                        name: definition.name.clone(),
                        err,
                    });
                }
            },
            None => {
                if let Some(default_value_id) = definition.default_value_id {
                    variables[definition.id] = VariableValueRecord::DefaultValue(default_value_id);
                } else if definition.ty_record.wrapping.is_required() {
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

impl Binder<'_, '_> {
    pub(super) fn bind_variable_definitions(
        &mut self,
        variables: cynic_parser::executable::Iter<'_, cynic_parser::executable::VariableDefinition<'_>>,
    ) -> BindResult<()> {
        let mut seen_names = HashSet::new();

        for variable in variables {
            let name = variable.name().to_string();
            let name_location = self.parsed_operation.span_to_location(variable.name_span());

            if seen_names.contains(&name) {
                return Err(BindError::DuplicateVariable {
                    name,
                    location: name_location,
                });
            }
            seen_names.insert(name.clone());

            let mut ty = self.convert_type(&name, variable.ty())?;

            match variable.default_value() {
                Some(value) if !value.is_null() => {
                    if ty.wrapping.is_list() {
                        ty.wrapping = ty.wrapping.wrap_list_non_null();
                    } else {
                        ty.wrapping = Wrapping::new(true);
                    }
                }
                _ => (),
            }

            let ty = ty.walk(self.schema);
            let default_value = variable
                .default_value()
                .map(|value| coerce_variable_default_value(self, ty, value))
                .transpose()?;

            self.variable_definition_in_use.push(false);
            self.operation.variable_definitions.push(VariableDefinitionRecord {
                name,
                name_location,
                default_value_id: default_value,
                ty_record: ty.into(),
            });
        }

        Ok(())
    }

    pub(super) fn validate_all_variables_used(&self) -> BindResult<()> {
        for (variable, in_use) in self
            .operation
            .variable_definitions
            .iter()
            .zip(&self.variable_definition_in_use)
        {
            if !in_use {
                return Err(BindError::UnusedVariable {
                    name: variable.name.clone(),
                    operation: self.error_operation_name.clone(),
                    location: variable.name_location,
                });
            }
        }

        Ok(())
    }

    fn convert_type(
        &self,
        variable_name: &str,
        ty: cynic_parser::executable::Type<'_>,
    ) -> BindResult<schema::TypeRecord> {
        use cynic_parser::common::WrappingType;

        let location = ty.span();

        let definition = self
            .schema
            .definition_by_name(ty.name())
            .ok_or_else(|| BindError::UnknownType {
                name: ty.name().to_string(),
                span: location,
            })?;

        if !matches!(
            definition,
            Definition::Enum(_) | Definition::Scalar(_) | Definition::InputObject(_)
        ) {
            return Err(BindError::InvalidVariableType {
                name: variable_name.to_string(),
                ty: definition.name().to_string(),
                span: location,
            });
        }

        let mut wrapping = schema::Wrapping::default();
        let wrappers = ty.wrappers().collect::<Vec<_>>();
        // from innermost to outermost
        for wrapper in wrappers.into_iter().rev() {
            wrapping = match wrapper {
                WrappingType::NonNull => wrapping.wrap_non_null(),
                WrappingType::List => wrapping.wrap_list(),
            };
        }

        Ok(schema::TypeRecord {
            definition_id: definition.id(),
            wrapping,
        })
    }
}
