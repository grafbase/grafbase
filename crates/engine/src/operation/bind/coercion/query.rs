use engine_value::{ConstValue, Name, Value};
use id_newtypes::IdRange;
use schema::{
    Definition, DefinitionId, EnumDefinition, InputObjectDefinition, InputValueDefinitionId, ListWrapping,
    ScalarDefinition, ScalarType, Type, TypeRecord, Wrapping,
};
use walker::Walk;

use super::super::Binder;
use super::{
    error::InputValueError,
    path::{value_path_to_string, ValuePathSegment},
};
use crate::operation::{Location, QueryInputValueId, QueryInputValueRecord};

pub fn coerce_variable_default_value<'schema>(
    binder: &mut Binder<'schema, '_>,
    location: Location,
    ty: Type<'schema>,
    value: ConstValue,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        binder,
        location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
        is_default_value: true,
    };
    let value = ctx.coerce_input_value(ty, value.into())?;
    Ok(ctx.input_values.push_value(value))
}

pub fn coerce_query_value<'schema>(
    binder: &mut Binder<'schema, '_>,
    location: Location,
    ty: Type<'schema>,
    value: Value,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        binder,
        location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
        is_default_value: false,
    };
    let value = ctx.coerce_input_value(ty, value)?;
    Ok(ctx.input_values.push_value(value))
}

struct QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    binder: &'binder mut Binder<'schema, 'parsed>,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, QueryInputValueRecord)>>,
    is_default_value: bool,
}

impl<'schema, 'parsed> std::ops::Deref for QueryValueCoercionContext<'_, 'schema, 'parsed> {
    type Target = Binder<'schema, 'parsed>;

    fn deref(&self) -> &Self::Target {
        self.binder
    }
}

impl std::ops::DerefMut for QueryValueCoercionContext<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.binder
    }
}

impl<'schema> QueryValueCoercionContext<'_, 'schema, '_> {
    fn variable_ref(&mut self, name: Name, ty: Type<'schema>) -> Result<QueryInputValueRecord, InputValueError> {
        if self.is_default_value {
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

        let variable_ty = self.variable_definitions[id].ty_record;
        if !are_type_compatibles(variable_ty, ty.into()) {
            return Err(InputValueError::IncorrectVariableType {
                name: name.to_string(),
                variable_ty: variable_ty.walk(self.schema).to_string(),
                actual_ty: ty.to_string(),
                location: self.location,
                path: self.path(),
            });
        }

        self.variable_definition_in_use[id] = true;

        Ok(QueryInputValueRecord::Variable(id.into()))
    }

    fn coerce_input_value(
        &mut self,
        ty: Type<'schema>,
        value: Value,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_array() && !value.is_null() && !value.is_variable() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = QueryInputValueRecord::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_wrapped_value(ty, value)
    }

    fn coerce_wrapped_value(
        &mut self,
        mut ty: Type<'schema>,
        value: Value,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        if let Value::Variable(name) = value {
            return self.variable_ref(name, ty);
        }

        let Some(list_wrapping) = ty.pop_list_wrapping() else {
            return self.coerce_named_type(ty, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: ty.wrapped_by(list_wrapping).to_string(),
                path: self.path(),
                location: self.location,
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(QueryInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_wrapped_value(ty, value)?;
                    self.value_path.pop();
                }
                Ok(QueryInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: ty.wrapped_by(list_wrapping).to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: Type<'schema>, value: Value) -> Result<QueryInputValueRecord, InputValueError> {
        if let Value::Variable(name) = value {
            return self.variable_ref(name, ty);
        }

        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: ty.to_string(),
                    path: self.path(),
                    location: self.location,
                });
            }
            return Ok(QueryInputValueRecord::Null);
        }

        // At this point the definition should be accessible, otherwise the input value should have
        // been rejected earlier.
        match ty.definition() {
            Definition::Scalar(scalar) => self.coerce_scalar(scalar, value),
            Definition::Enum(r#enum) => self.coerce_enum(r#enum, value),
            Definition::InputObject(input_object) => self.coerce_input_object(input_object, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_object(
        &mut self,
        input_object: InputObjectDefinition<'schema>,
        value: Value,
    ) -> Result<QueryInputValueRecord, InputValueError> {
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
            if input_field.is_inaccessible() {
                continue;
            }
            match fields.swap_remove(input_field.name()) {
                None => {
                    if let Some(default_value_id) = input_field.as_ref().default_value_id {
                        fields_buffer.push((input_field.id, QueryInputValueRecord::DefaultValue(default_value_id)));
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
                    let value = self.coerce_input_value(input_field.ty(), value)?;
                    fields_buffer.push((input_field.id, value));
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
        Ok(QueryInputValueRecord::InputObject(ids))
    }

    fn coerce_enum(
        &mut self,
        r#enum: EnumDefinition<'schema>,
        value: Value,
    ) -> Result<QueryInputValueRecord, InputValueError> {
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

        let Some(value) = r#enum.find_value_by_name(name).filter(|value| !value.is_inaccessible()) else {
            return Err(InputValueError::UnknownEnumValue {
                r#enum: r#enum.name().to_string(),
                value: name.to_string(),
                location: self.location,
                path: self.path(),
            });
        };

        Ok(QueryInputValueRecord::EnumValue(value.id))
    }

    fn coerce_scalar(
        &mut self,
        scalar: ScalarDefinition<'schema>,
        value: Value,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::JSON) => Ok(match value {
                Value::Null => QueryInputValueRecord::Null,
                Value::Number(n) => {
                    if let Some(n) = n.as_f64() {
                        QueryInputValueRecord::Float(n)
                    } else if let Some(n) = n.as_i64() {
                        QueryInputValueRecord::BigInt(n)
                    } else {
                        QueryInputValueRecord::U64(n.as_u64().unwrap())
                    }
                }
                Value::String(s) => QueryInputValueRecord::String(s),
                Value::Boolean(b) => QueryInputValueRecord::Boolean(b),
                Value::List(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        self.input_values[id] = self.coerce_scalar(scalar, value)?;
                    }
                    QueryInputValueRecord::List(ids)
                }
                Value::Object(fields) => {
                    let ids = self.input_values.reserve_map(fields.len());
                    for ((name, value), id) in fields.into_iter().zip(ids) {
                        let key = name.into_string();
                        self.input_values[id] = (key, self.coerce_scalar(scalar, value)?);
                    }
                    QueryInputValueRecord::Map(ids)
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
                Ok(QueryInputValueRecord::Int(value))
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
                Ok(QueryInputValueRecord::BigInt(value))
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
                Ok(QueryInputValueRecord::Float(value))
            }
            (Value::String(value), ScalarType::String) => Ok(QueryInputValueRecord::String(value)),
            (Value::Boolean(value), ScalarType::Boolean) => Ok(QueryInputValueRecord::Boolean(value)),
            (Value::Binary(_), _) => unreachable!("Parser doesn't generate bytes, nor do variables."),
            (Value::Variable(name), _) => self.variable_ref(
                name,
                TypeRecord {
                    definition_id: DefinitionId::Scalar(scalar.id),
                    wrapping: Wrapping::nullable(),
                }
                .walk(self.schema),
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

/// Determines whether a variable is compatible with the expected type
fn are_type_compatibles(left: TypeRecord, right: TypeRecord) -> bool {
    left.definition_id == right.definition_id
            // if not a list, the current type can be coerced into the proper list wrapping.
            && (!left.wrapping.is_list()
                || left.wrapping.list_wrappings().len() == right.wrapping.list_wrappings().len())
            && (right.wrapping.is_nullable() || left.wrapping.is_required())
}
