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

    fn path(&self) -> String {
        value_path_to_string(self.schema, &self.value_path)
    }
}
