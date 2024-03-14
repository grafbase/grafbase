use engine_value::{Name, Value};
use id_newtypes::IdRange;
use schema::{
    Definition, EnumWalker, InputObjectWalker, InputValueDefinitionId, ListWrapping, ScalarType, ScalarWalker, Type,
    Wrapping,
};

use crate::request::{BoundFieldId, Location, OpInputValue, OpInputValueId};

use super::super::Binder;
use super::{
    error::InputValueError,
    path::{value_path_to_string, ValuePathSegment},
};

struct ValueCoercionContext<'a, 'b> {
    binder: &'a mut Binder<'b>,
    bound_field_id: BoundFieldId,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, OpInputValue)>>,
}

impl<'a, 'b> std::ops::Deref for ValueCoercionContext<'a, 'b> {
    type Target = Binder<'b>;

    fn deref(&self) -> &Self::Target {
        self.binder
    }
}

impl<'a, 'b> std::ops::DerefMut for ValueCoercionContext<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.binder
    }
}

pub fn coerce_value(
    binder: &mut Binder<'_>,
    bound_field_id: BoundFieldId,
    location: Location,
    ty: Type,
    value: Value,
) -> Result<OpInputValueId, InputValueError> {
    let mut ctx = ValueCoercionContext {
        binder,
        bound_field_id,
        location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
    };
    let value = ctx.coerce_input_value(ty, value)?;
    Ok(ctx.input_values.push_value(value))
}

impl<'a, 'b> ValueCoercionContext<'a, 'b> {
    fn variable_ref(&mut self, name: Name, ty: Type) -> Result<OpInputValue, InputValueError> {
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

        let variable_ty = self.variable_definitions[id].r#type;
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
        if self.variable_definitions[id].used_by.last() != Some(&self.bound_field_id) {
            self.binder.variable_definitions[id].used_by.push(self.bound_field_id);
        }

        Ok(OpInputValue::Ref(self.variable_definitions[id].future_input_value_id))
    }

    fn coerce_input_value(&mut self, ty: Type, value: Value) -> Result<OpInputValue, InputValueError> {
        if ty.wrapping.is_list() && !value.is_array() && !value.is_null() && !value.is_variable() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = OpInputValue::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_list(ty, value)
    }

    fn coerce_list(&mut self, mut ty: Type, value: Value) -> Result<OpInputValue, InputValueError> {
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
            (Value::Null, ListWrapping::NullableList) => Ok(OpInputValue::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_list(ty, value)?;
                    self.value_path.pop();
                }
                Ok(OpInputValue::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: Type, value: Value) -> Result<OpInputValue, InputValueError> {
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
            } else {
                return Ok(OpInputValue::Null);
            }
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
        input_object: InputObjectWalker<'_>,
        value: Value,
    ) -> Result<OpInputValue, InputValueError> {
        let Value::Object(mut fields) = value else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: value.into(),
                path: self.path(),
                location: self.location,
            });
        };

        let mut buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();
        for input_field in input_object.input_fields() {
            match fields.swap_remove(input_field.name()) {
                None => {
                    if let Some(default_value_id) = input_field.as_ref().default_value {
                        buffer.push((input_field.id(), OpInputValue::SchemaRef(default_value_id)));
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
                    buffer.push((
                        input_field.id(),
                        self.coerce_input_value(*input_field.ty().as_ref(), value)?,
                    ));
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
        let ids = self.input_values.push_input_object(buffer.drain(..));
        self.input_fields_buffer_pool.push(buffer);
        Ok(OpInputValue::InputObject(ids))
    }

    fn coerce_enum(&mut self, r#enum: EnumWalker<'_>, value: Value) -> Result<OpInputValue, InputValueError> {
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

        Ok(OpInputValue::EnumValue(id))
    }

    fn coerce_scalar(&mut self, scalar: ScalarWalker<'_>, value: Value) -> Result<OpInputValue, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::JSON) => Ok(match value {
                Value::Null => OpInputValue::Null,
                Value::Number(n) => {
                    if let Some(n) = n.as_f64() {
                        OpInputValue::Float(n)
                    } else if let Some(n) = n.as_i64() {
                        OpInputValue::BigInt(n)
                    } else {
                        OpInputValue::U64(n.as_u64().unwrap())
                    }
                }
                Value::String(s) => OpInputValue::String(s.into_boxed_str()),
                Value::Boolean(b) => OpInputValue::Boolean(b),
                Value::List(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        self.input_values[id] = self.coerce_scalar(scalar, value)?;
                    }
                    OpInputValue::List(ids)
                }
                Value::Object(fields) => {
                    // should be static?
                    let empty: Box<str> = String::new().into_boxed_str();
                    let ids = self.input_values.reserve_map(empty, fields.len());
                    for ((name, value), id) in fields.into_iter().zip(ids) {
                        let key = name.to_string().into_boxed_str();
                        self.input_values[id] = (key, self.coerce_scalar(scalar, value)?);
                    }
                    OpInputValue::Map(ids)
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
                Ok(OpInputValue::Int(value))
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
                Ok(OpInputValue::BigInt(value))
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
                Ok(OpInputValue::Float(value))
            }
            (Value::String(value), ScalarType::String) => Ok(OpInputValue::String(value.into_boxed_str())),
            (Value::Boolean(value), ScalarType::Boolean) => Ok(OpInputValue::Boolean(value)),
            (Value::Binary(_), _) => unreachable!("Parser doesn't generate bytes, nor do variables."),
            (Value::Variable(name), _) => self.variable_ref(
                name,
                Type {
                    inner: Definition::Scalar(scalar.id()),
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

    fn path(&self) -> String {
        value_path_to_string(self.schema, &self.value_path)
    }
}
