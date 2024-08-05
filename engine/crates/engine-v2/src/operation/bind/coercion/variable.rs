use engine_value::ConstValue;
use id_newtypes::IdRange;
use schema::{
    Definition, EnumDefinitionWalker, InputObjectDefinitionWalker, InputValueDefinitionId, ListWrapping,
    ScalarDefinitionWalker, ScalarType, Schema, Type,
};

use crate::operation::{Location, VariableDefinition, VariableInputValue, VariableInputValueId, VariableInputValues};

use super::{
    error::InputValueError,
    path::{value_path_to_string, ValuePathSegment},
};

pub fn coerce_variable(
    schema: &Schema,
    input_values: &mut VariableInputValues,
    definition: &VariableDefinition,
    value: ConstValue,
) -> Result<VariableInputValueId, InputValueError> {
    let mut ctx = VariableCoercionContext {
        schema,
        input_values,
        location: definition.name_location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
    };
    let value = ctx.coerce_input_value(definition.ty, value)?;
    Ok(input_values.push_value(value))
}

struct VariableCoercionContext<'a> {
    schema: &'a Schema,
    input_values: &'a mut VariableInputValues,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, VariableInputValue)>>,
}

impl<'a> VariableCoercionContext<'a> {
    fn coerce_input_value(&mut self, ty: Type, value: ConstValue) -> Result<VariableInputValue, InputValueError> {
        if ty.wrapping.is_list() && !value.is_array() && !value.is_null() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = VariableInputValue::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_list(ty, value)
    }

    fn coerce_list(&mut self, mut ty: Type, value: ConstValue) -> Result<VariableInputValue, InputValueError> {
        let Some(list_wrapping) = ty.wrapping.pop_list_wrapping() else {
            return self.coerce_named_type(ty, value);
        };

        match (value, list_wrapping) {
            (ConstValue::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
                location: self.location,
            }),
            (ConstValue::Null, ListWrapping::NullableList) => Ok(VariableInputValue::Null),
            (ConstValue::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_list(ty, value)?;
                    self.value_path.pop();
                }
                Ok(VariableInputValue::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: Type, value: ConstValue) -> Result<VariableInputValue, InputValueError> {
        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: self.schema.walk(ty).to_string(),
                    path: self.path(),
                    location: self.location,
                });
            }
            return Ok(VariableInputValue::Null);
        }

        match ty.inner {
            Definition::Scalar(scalar) => self.coerce_scalar(self.schema.walk(scalar), value),
            Definition::Enum(r#enum) => self.coerce_enum(self.schema.walk(r#enum), value),
            Definition::InputObject(input_object) => self.coerce_input_objet(self.schema.walk(input_object), value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_objet(
        &mut self,
        input_object: InputObjectDefinitionWalker<'_>,
        value: ConstValue,
    ) -> Result<VariableInputValue, InputValueError> {
        let ConstValue::Object(mut fields) = value else {
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
                    if let Some(default_value_id) = input_field.as_ref().default_value {
                        fields_buffer.push((input_field.id(), VariableInputValue::DefaultValue(default_value_id)));
                    } else if input_field.ty().wrapping().is_required() {
                        return Err(InputValueError::UnexpectedNull {
                            expected: input_field.ty().to_string(),
                            path: self.path(),
                            location: self.location,
                        });
                    }
                }
                Some(value) => {
                    self.value_path.push(input_field.as_ref().name.into());
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
        Ok(VariableInputValue::InputObject(ids))
    }

    fn coerce_enum(
        &mut self,
        r#enum: EnumDefinitionWalker<'_>,
        value: ConstValue,
    ) -> Result<VariableInputValue, InputValueError> {
        let name = match &value {
            ConstValue::Enum(value) => value.as_str(),
            ConstValue::String(value) => value.as_str(),
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

        Ok(VariableInputValue::EnumValue(id))
    }

    fn coerce_scalar(
        &mut self,
        scalar: ScalarDefinitionWalker<'_>,
        value: ConstValue,
    ) -> Result<VariableInputValue, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::JSON) => Ok(match value {
                ConstValue::Null => VariableInputValue::Null,
                ConstValue::Number(n) => {
                    if let Some(n) = n.as_f64() {
                        VariableInputValue::Float(n)
                    } else if let Some(n) = n.as_i64() {
                        VariableInputValue::BigInt(n)
                    } else {
                        VariableInputValue::U64(n.as_u64().unwrap())
                    }
                }
                ConstValue::String(s) => VariableInputValue::String(s),
                ConstValue::Boolean(b) => VariableInputValue::Boolean(b),
                ConstValue::List(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        let value = self.coerce_scalar(scalar, value)?;
                        self.input_values[id] = value;
                    }
                    VariableInputValue::List(ids)
                }
                ConstValue::Object(fields) => {
                    let ids = self.input_values.reserve_map(fields.len());
                    for ((name, value), id) in fields.into_iter().zip(ids) {
                        let key = name.into_string();
                        self.input_values[id] = (key, self.coerce_scalar(scalar, value)?);
                    }
                    VariableInputValue::Map(ids)
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
            (ConstValue::Number(number), ScalarType::Int) => {
                let Some(value) = number.as_i64().and_then(|n| i32::try_from(n).ok()) else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(VariableInputValue::Int(value))
            }
            (ConstValue::Number(number), ScalarType::BigInt) => {
                let Some(value) = number.as_i64() else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(VariableInputValue::BigInt(value))
            }
            (ConstValue::Number(number), ScalarType::Float) => {
                let Some(value) = number.as_f64() else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(VariableInputValue::Float(value))
            }
            (ConstValue::String(value), ScalarType::String) => Ok(VariableInputValue::String(value)),
            (ConstValue::Boolean(value), ScalarType::Boolean) => Ok(VariableInputValue::Boolean(value)),
            (ConstValue::Binary(_), _) => unreachable!("Parser doesn't generate bytes, nor do variables."),
            (actual, _) => Err(InputValueError::IncorrectScalarType {
                actual: actual.into(),
                expected: scalar.name().to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    fn path(&self) -> String {
        value_path_to_string(self.schema, &self.value_path)
    }
}
