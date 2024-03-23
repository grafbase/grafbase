mod error;
mod path;

use crate::{
    Definition, EnumWalker, InputObjectWalker, InputValueDefinitionId, ScalarType, ScalarWalker, Schema,
    SchemaInputValue, SchemaInputValueId, SchemaInputValues, StringId, Type,
};
pub use error::*;
use federated_graph::Value;
use id_newtypes::IdRange;
use path::*;
use wrapping::ListWrapping;

pub(super) struct InputValueCoercer<'a> {
    schema: &'a Schema,
    input_values: &'a mut SchemaInputValues,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, SchemaInputValue)>>,
}

impl<'a> InputValueCoercer<'a> {
    pub fn new(schema: &'a Schema, input_values: &'a mut SchemaInputValues) -> Self {
        Self {
            schema,
            input_values,
            value_path: Vec::new(),
            input_fields_buffer_pool: Vec::new(),
        }
    }

    pub fn coerce(&mut self, ty: Type, value: Value) -> Result<SchemaInputValueId, InputValueError> {
        let value = self.coerce_input_value(ty, value)?;
        Ok(self.input_values.push_value(value))
    }

    fn coerce_input_value(&mut self, ty: Type, value: Value) -> Result<SchemaInputValue, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = SchemaInputValue::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_list(ty, value)
    }

    fn coerce_list(&mut self, mut ty: Type, value: Value) -> Result<SchemaInputValue, InputValueError> {
        let Some(list_wrapping) = ty.wrapping.pop_list_wrapping() else {
            return self.coerce_named_type(ty, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(SchemaInputValue::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_vec().into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_list(ty, value)?;
                    self.value_path.pop();
                }
                Ok(SchemaInputValue::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.schema.walk(ty.wrapped_by(list_wrapping)).to_string(),
                path: self.path(),
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: Type, value: Value) -> Result<SchemaInputValue, InputValueError> {
        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: self.schema.walk(ty).to_string(),
                    path: self.path(),
                });
            }
            return Ok(SchemaInputValue::Null);
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
    ) -> Result<SchemaInputValue, InputValueError> {
        let Value::Object(fields) = value else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: value.into(),
                path: self.path(),
            });
        };

        let mut fields = fields
            .into_vec()
            .into_iter()
            .map(|(id, value)| (id, Some(value)))
            .collect::<Vec<_>>();
        fields.sort_unstable_by_key(|(id, _)| *id);
        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();
        for input_field in input_object.input_fields() {
            match fields.binary_search_by_key(&input_field.as_ref().name, |(id, _)| StringId::from(*id)) {
                Ok(i) => {
                    let value = std::mem::take(&mut fields[i].1).unwrap();
                    self.value_path.push(input_field.as_ref().name.into());
                    let value = self.coerce_input_value(input_field.ty().into(), value)?;
                    fields_buffer.push((input_field.id(), value));
                    self.value_path.pop();
                }
                Err(_) => {
                    if let Some(default_value_id) = input_field.as_ref().default_value {
                        fields_buffer.push((input_field.id(), self.schema[default_value_id]));
                    } else if input_field.ty().wrapping().is_required() {
                        return Err(InputValueError::UnexpectedNull {
                            expected: input_field.ty().to_string(),
                            path: self.path(),
                        });
                    }
                }
            }
        }
        if let Some((id, _)) = fields
            .into_iter()
            .filter_map(|(id, maybe_value)| Some((id, maybe_value?)))
            .next()
        {
            return Err(InputValueError::UnknownInputField {
                input_object: input_object.name().to_string(),
                name: self.schema[StringId::from(id)].to_string(),
                path: self.path(),
            });
        }
        let ids = self.input_values.append_input_object(&mut fields_buffer);
        self.input_fields_buffer_pool.push(fields_buffer);
        Ok(SchemaInputValue::InputObject(ids))
    }

    fn coerce_enum(&mut self, r#enum: EnumWalker<'_>, value: Value) -> Result<SchemaInputValue, InputValueError> {
        let name = match &value {
            Value::EnumValue(id) => &self.schema[StringId::from(*id)],
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: r#enum.name().to_owned(),
                    actual: value.into(),
                    path: self.path(),
                })
            }
        };

        let Some(id) = r#enum.find_value_by_name(name) else {
            return Err(InputValueError::UnknownEnumValue {
                r#enum: r#enum.name().to_string(),
                value: name.to_string(),
                path: self.path(),
            });
        };

        Ok(SchemaInputValue::EnumValue(id))
    }

    fn coerce_scalar(&mut self, scalar: ScalarWalker<'_>, value: Value) -> Result<SchemaInputValue, InputValueError> {
        match scalar.as_ref().ty {
            ScalarType::String => match value {
                Value::String(id) => Some(id.into()),
                _ => None,
            }
            .map(SchemaInputValue::String),
            ScalarType::Float => match value {
                Value::Int(n) => Some(n as f64),
                Value::Float(f) => Some(f),
                _ => None,
            }
            .map(SchemaInputValue::Float),
            ScalarType::Int => match value {
                Value::Int(n) => {
                    let n = i32::try_from(n).map_err(|_| InputValueError::IncorrectScalarValue {
                        actual: n.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                    })?;
                    Some(n)
                }
                _ => None,
            }
            .map(SchemaInputValue::Int),
            ScalarType::BigInt => match value {
                Value::Int(n) => Some(n),
                _ => None,
            }
            .map(SchemaInputValue::BigInt),
            ScalarType::Boolean => match value {
                Value::Boolean(b) => Some(b),
                _ => None,
            }
            .map(SchemaInputValue::Boolean),
            ScalarType::JSON => {
                return Ok(match value {
                    Value::Null => SchemaInputValue::Null,
                    Value::String(id) => SchemaInputValue::String(id.into()),
                    Value::Int(n) => SchemaInputValue::BigInt(n),
                    Value::Float(f) => SchemaInputValue::Float(f),
                    Value::Boolean(b) => SchemaInputValue::Boolean(b),
                    Value::EnumValue(id) => SchemaInputValue::String(id.into()),
                    Value::Object(fields) => {
                        let ids = self.input_values.reserve_map(fields.len());
                        for ((name, value), id) in fields.into_vec().into_iter().zip(ids) {
                            self.input_values[id] = (name.into(), self.coerce_scalar(scalar, value)?);
                        }
                        SchemaInputValue::Map(ids)
                    }
                    Value::List(list) => {
                        let ids = self.input_values.reserve_list(list.len());
                        for (value, id) in list.into_vec().into_iter().zip(ids) {
                            let value = self.coerce_scalar(scalar, value)?;
                            self.input_values[id] = value;
                        }
                        SchemaInputValue::List(ids)
                    }
                })
            }
        }
        .ok_or_else(|| InputValueError::IncorrectScalarType {
            actual: value.into(),
            expected: scalar.name().to_string(),
            path: self.path(),
        })
    }

    fn path(&self) -> String {
        value_path_to_string(self.schema, &self.value_path)
    }
}
