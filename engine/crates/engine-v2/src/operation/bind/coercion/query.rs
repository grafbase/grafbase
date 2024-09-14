use engine_value::{ConstValue, Name, Value};
use id_newtypes::IdRange;
use schema::{
    DefinitionId, EnumDefinition, InputObjectDefinition, InputValueDefinitionId, ListWrapping, ScalarDefinition,
    ScalarType, TypeRecord, Wrapping,
};

use super::super::Binder;
use super::{
    error::InputValueError,
    path::{value_path_to_string, ValuePathSegment},
};
use crate::operation::{FieldId, Location, QueryInputValue, QueryInputValueId};

/// Coerces the default value of a variable into a `QueryInputValueId`.
///
/// This function is typically used when handling variable default values in
/// GraphQL queries. It takes the binder, the location in the AST (Abstract
/// Syntax Tree), the type record of the input value, and the default value
/// itself. The coercion process involves validating the value against the
/// expected type, handling any necessary type conversion, and returning
/// the corresponding `QueryInputValueId`.
///
/// # Parameters
///
/// - `binder`: A mutable reference to the Binder for managing variable context.
/// - `location`: A `Location` indicating where in the query the value resides.
/// - `ty`: The `TypeRecord` representing the expected type of the value.
/// - `value`: The default value as a `ConstValue` that needs coercion.
///
/// # Returns
///
/// - Ok if the coercion is successful, or an `InputValueError` if coercion fails.
///
/// # Errors
///
/// Possible errors include `InputValueError::UnknownVariable`,
/// `InputValueError::IncorrectVariableType`, and others related to
/// mismatched types or null values.
pub fn coerce_variable_default_value(
    binder: &mut Binder<'_, '_>,
    location: Location,
    ty: TypeRecord,
    value: ConstValue,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        binder,
        field_id: None,
        location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
    };
    let value = ctx.coerce_input_value(ty, value.into())?;
    Ok(ctx.input_values.push_value(value))
}

pub fn coerce_query_value(
    binder: &mut Binder<'_, '_>,
    field_id: FieldId,
    location: Location,
    ty: TypeRecord,
    value: Value,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        binder,
        field_id: Some(field_id),
        location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
    };
    let value = ctx.coerce_input_value(ty, value)?;
    Ok(ctx.input_values.push_value(value))
}

struct QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    binder: &'binder mut Binder<'schema, 'parsed>,
    field_id: Option<FieldId>,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, QueryInputValue)>>,
}

impl<'binder, 'schema, 'parsed> std::ops::Deref for QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    type Target = Binder<'schema, 'parsed>;

    fn deref(&self) -> &Self::Target {
        self.binder
    }
}

impl<'binder, 'schema, 'parsed> std::ops::DerefMut for QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.binder
    }
}

impl<'binder, 'schema, 'parsed> QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    /// Retrieves a reference to a variable by name and type, coercing it into a `QueryInputValue`.
    ///
    /// This function attempts to find the variable definition in the context and checks if its type
    /// is compatible with the given type. If the variable is not found, or if its type is incorrect,
    /// an appropriate `InputValueError` is returned.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the variable to reference.
    /// - `ty`: The expected type of the variable as a `TypeRecord`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` containing the variable ID if the coercion is successful.
    /// Returns an `InputValueError` if the variable definition is unknown or the type is incorrect.
    ///
    /// # Errors
    ///
    /// Possible errors include:
    ///
    /// - `InputValueError::VariableDefaultValueReliesOnAnotherVariable`: if the variable ID is not
    ///   available.
    /// - `InputValueError::UnknownVariable`: if the variable cannot be found in the current context.
    /// - `InputValueError::IncorrectVariableType`: if the variable type is not compatible with the
    ///   specified type.
    fn variable_ref(&mut self, name: Name, ty: TypeRecord) -> Result<QueryInputValue, InputValueError> {
        // field_id is not provided for variable default values.
        let Some(field_id) = self.field_id else {
            return Err(InputValueError::VariableDefaultValueReliesOnAnotherVariable {
                name: name.to_string(),
                location: self.location,
                path: self.path(),
            });
        };

        let Some(id) = self
            .variable_definitions
            .iter()
            .position(|variable| variable.name == name)
        else {
            return Err(InputValueError::UnknownVariable {
                name: name.to_string(),
                location: self.location,
                path: self.path(),
            });
        };

        let variable_ty = self.variable_definitions[id].ty;
        if !variable_ty.is_compatible_with(ty) {
            return Err(InputValueError::IncorrectVariableType {
                name: name.to_string(),
                variable_ty: self.schema.walk(variable_ty).to_string(),
                actual_ty: self.schema.walk(ty).to_string(),
                location: self.location,
                path: self.path(),
            });
        }

        // This function is called during the binding where we create the BoundFieldIds
        // sequentially. So we're always processing the last BoundFieldId and this array is always
        // sorted.
        if self.variable_definitions[id].used_by.last() != self.field_id.as_ref() {
            self.binder.variable_definitions[id].used_by.push(field_id);
        }

        Ok(QueryInputValue::Variable(id.into()))
    }

    /// Coerces an input value into a `QueryInputValue`, validating its type against the expected `TypeRecord`.
    ///
    /// This function takes a value and attempts to coerce it to the expected type defined in `ty`.
    /// It handles types including lists and named types, ensuring that the coercion is valid and
    /// raising errors if the provided value does not match the expected type.
    ///
    /// # Parameters
    ///
    /// - `ty`: The `TypeRecord` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include type mismatches and unexpected null values.
    fn coerce_input_value(&mut self, ty: TypeRecord, value: Value) -> Result<QueryInputValue, InputValueError> {
        if ty.wrapping.is_list() && !value.is_array() && !value.is_null() && !value.is_variable() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = QueryInputValue::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_list(ty, value)
    }

    /// Coerces an input value into a list of `QueryInputValue`, validating its type against the expected `TypeRecord`.
    ///
    /// This function takes a value and attempts to coerce it to a list type defined in `ty`.
    /// It handles scenarios where the expected type is a list and ensures that the coercion
    /// is valid and raises errors if the provided value does not match the expected type.
    ///
    /// # Parameters
    ///
    /// - `ty`: The `TypeRecord` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include type mismatches and unexpected null values.
    fn coerce_list(&mut self, mut ty: TypeRecord, value: Value) -> Result<QueryInputValue, InputValueError> {
        if let Value::Variable(name) = value {
            return self.variable_ref(name, ty);
        }

        let Some(list_wrapping) = ty.wrapping.pop_list_wrapping() else {
            return self.coerce_named_type(ty, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
                location: self.location,
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(QueryInputValue::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_list(ty, value)?;
                    self.value_path.pop();
                }
                Ok(QueryInputValue::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    /// Coerces an input value into a `QueryInputValue` given a named type, validating its type against the expected `TypeRecord`.
    ///
    /// This function takes a value and attempts to coerce it to the expected type defined in `ty`.
    /// It handles named types, ensuring that the coercion is valid and raises errors if the provided value
    /// does not match the expected type.
    ///
    /// # Parameters
    ///
    /// - `ty`: The `TypeRecord` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include mismatched types and unexpected null values.
    fn coerce_named_type(&mut self, ty: TypeRecord, value: Value) -> Result<QueryInputValue, InputValueError> {
        if let Value::Variable(name) = value {
            return self.variable_ref(name, ty);
        }

        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: self.schema.walk(ty).to_string(),
                    path: self.path(),
                    location: self.location,
                });
            }

            return Ok(QueryInputValue::Null);
        }

        match ty.definition_id {
            DefinitionId::Scalar(scalar) => self.coerce_scalar(self.schema.walk(scalar), value),
            DefinitionId::Enum(r#enum) => self.coerce_enum(self.schema.walk(r#enum), value),
            DefinitionId::InputObject(input_object) => self.coerce_input_objet(self.schema.walk(input_object), value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    /// Coerces an input value into a `QueryInputValue`, validating its type against the expected `TypeRecord`.
    ///
    /// This function takes a value and attempts to coerce it to the expected type defined in `input_object`.
    /// It handles scenarios where the value is expected to be an object, ensuring that the coercion
    /// is valid and raising errors if the provided value does not match the expected type.
    ///
    /// # Parameters
    ///
    /// - `input_object`: The `InputObjectDefinition` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include:
    ///
    /// - `InputValueError::MissingObject`: if the value is not an object when an object is expected.
    /// - `InputValueError::UnknownInputField`: if an input field is provided that does not exist in the input object.
    /// - `InputValueError::UnexpectedNull`: if a required field has a null value.
    fn coerce_input_objet(
        &mut self,
        input_object: InputObjectDefinition<'_>,
        value: Value,
    ) -> Result<QueryInputValue, InputValueError> {
        let Value::Object(mut fields) = value else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: value.into(),
                path: self.path(),
                location: self.location,
            });
        };

        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();
        for input_field in input_object.input_fields() {
            match fields.swap_remove(input_field.name()) {
                None => {
                    if let Some(default_value_id) = input_field.as_ref().default_value_id {
                        fields_buffer.push((input_field.id(), QueryInputValue::DefaultValue(default_value_id)));
                    } else if input_field.ty().wrapping.is_required() {
                        return Err(InputValueError::UnexpectedNull {
                            expected: input_field.ty().to_string(),
                            path: self.path(),
                            location: self.location,
                        });
                    }
                }
                Some(value) => {
                    self.value_path.push(input_field.as_ref().name_id.into());
                    let value = self.coerce_input_value(input_field.ty().into(), value)?;
                    fields_buffer.push((input_field.id(), value));
                    self.value_path.pop();
                }
            }
        }
        if let Some(name) = fields.keys().next() {
            return Err(InputValueError::UnknownInputField {
                input_object: input_object.name().to_string(),
                name: name.to_string(),
                path: self.path(),
                location: self.location,
            });
        }
        let ids = self.input_values.append_input_object(&mut fields_buffer);
        self.input_fields_buffer_pool.push(fields_buffer);
        Ok(QueryInputValue::InputObject(ids))
    }

    /// Coerces an input value into a `QueryInputValue`, validating its type against the expected `EnumDefinition`.
    ///
    /// This function takes a value and attempts to coerce it to the expected enum type defined in `r#enum`.
    /// It checks if the provided value matches one of the defined enum variants and raises errors
    /// if the provided value does not match the expected type.
    ///
    /// # Parameters
    ///
    /// - `r#enum`: The `EnumDefinition` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include:
    ///
    /// - `InputValueError::IncorrectEnumValueType`: if the value does not match an enum variant.
    /// - `InputValueError::UnknownEnumValue`: if the enum value is not defined in the enum variants.
    fn coerce_enum(&mut self, r#enum: EnumDefinition<'_>, value: Value) -> Result<QueryInputValue, InputValueError> {
        let name = match &value {
            Value::Enum(value) => value.as_str(),
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: r#enum.name().to_owned(),
                    actual: value.into(),
                    path: self.path(),
                    location: self.location,
                })
            }
        };

        let Some(id) = r#enum.find_value_by_name(name) else {
            return Err(InputValueError::UnknownEnumValue {
                r#enum: r#enum.name().to_string(),
                value: name.to_string(),
                location: self.location,
                path: self.path(),
            });
        };

        Ok(QueryInputValue::EnumValue(id))
    }

    /// Coerces an input value into a `QueryInputValue`, validating its type against the expected `ScalarDefinition`.
    ///
    /// This function takes a value and attempts to coerce it to the expected scalar type defined in
    /// `scalar`. It handles different scalar types, including `JSON`, `Int`, `BigInt`, `Float`,
    /// `String`, and `Boolean`. If the provided value does not match the expected scalar type, an
    /// appropriate `InputValueError` is returned.
    ///
    /// # Parameters
    ///
    /// - `scalar`: The `ScalarDefinition` representing the expected type of the value.
    /// - `value`: The value to be coerced as a `Value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(QueryInputValue)` if coercion is successful. If coercion fails, it returns an
    /// `InputValueError`.
    ///
    /// # Errors
    ///
    /// Possible errors include:
    ///
    /// - `InputValueError::IncorrectScalarType`: if the actual value type does not match the expected scalar type.
    /// - `InputValueError::IncorrectScalarValue`: if the value does not correspond to the expected format for the scalar.
    fn coerce_scalar(
        &mut self,
        scalar: ScalarDefinition<'_>,
        value: Value,
    ) -> Result<QueryInputValue, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::JSON) => Ok(match value {
                Value::Null => QueryInputValue::Null,
                Value::Number(n) => {
                    if let Some(n) = n.as_f64() {
                        QueryInputValue::Float(n)
                    } else if let Some(n) = n.as_i64() {
                        QueryInputValue::BigInt(n)
                    } else {
                        QueryInputValue::U64(n.as_u64().unwrap())
                    }
                }
                Value::String(s) => QueryInputValue::String(s),
                Value::Boolean(b) => QueryInputValue::Boolean(b),
                Value::List(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        self.input_values[id] = self.coerce_scalar(scalar, value)?;
                    }
                    QueryInputValue::List(ids)
                }
                Value::Object(fields) => {
                    let ids = self.input_values.reserve_map(fields.len());
                    for ((name, value), id) in fields.into_iter().zip(ids) {
                        let key = name.into_string();
                        self.input_values[id] = (key, self.coerce_scalar(scalar, value)?);
                    }
                    QueryInputValue::Map(ids)
                }
                other => {
                    return Err(InputValueError::IncorrectScalarType {
                        actual: other.into(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                }
            }),
            (Value::Number(number), ScalarType::Int) => {
                let Some(value) = number.as_i64().and_then(|n| i32::try_from(n).ok()) else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(QueryInputValue::Int(value))
            }
            (Value::Number(number), ScalarType::BigInt) => {
                let Some(value) = number.as_i64() else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(QueryInputValue::BigInt(value))
            }
            (Value::Number(number), ScalarType::Float) => {
                let Some(value) = number.as_f64() else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(QueryInputValue::Float(value))
            }
            (Value::String(value), ScalarType::String) => Ok(QueryInputValue::String(value)),
            (Value::Boolean(value), ScalarType::Boolean) => Ok(QueryInputValue::Boolean(value)),
            (Value::Binary(_), _) => unreachable!("Parser doesn't generate bytes, nor do variables."),
            (Value::Variable(name), _) => self.variable_ref(
                name,
                TypeRecord {
                    definition_id: DefinitionId::Scalar(scalar.id()),
                    wrapping: Wrapping::nullable(),
                },
            ),
            (actual, _) => Err(InputValueError::IncorrectScalarType {
                actual: actual.into(),
                expected: scalar.name().to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    /// Retrieves the string representation of the current value path, which helps
    /// in error reporting and tracing the location of input values within the
    /// GraphQL query structure.
    ///
    /// # Returns
    ///
    /// A `String` that represents the path of the value being coerced. This
    /// path can be useful for debugging and understanding where a value
    /// was found in the query.
    fn path(&self) -> String {
        value_path_to_string(self.schema, &self.value_path)
    }
}
