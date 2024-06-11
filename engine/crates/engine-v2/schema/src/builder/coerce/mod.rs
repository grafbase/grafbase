mod error;
mod path;

use crate::{
    Definition, EnumId, Graph, InputObjectId, InputValueDefinitionId, ScalarId, ScalarType, SchemaInputValue,
    SchemaInputValueId, SchemaInputValues, StringId, Type,
};
pub use error::*;
use federated_graph::Value;
use id_newtypes::IdRange;
use path::*;
use wrapping::ListWrapping;

use super::BuildContext;

pub(super) struct InputValueCoercer<'a> {
    ctx: &'a BuildContext,
    graph: &'a Graph,
    input_values: &'a mut SchemaInputValues,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, SchemaInputValue)>>,
}

impl<'a> InputValueCoercer<'a> {
    pub fn new(ctx: &'a BuildContext, graph: &'a Graph, input_values: &'a mut SchemaInputValues) -> Self {
        Self {
            ctx,
            graph,
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
                expected: self.type_name(ty.wrapped_by(list_wrapping)),
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
                expected: self.type_name(ty.wrapped_by(list_wrapping)),
                path: self.path(),
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: Type, value: Value) -> Result<SchemaInputValue, InputValueError> {
        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: self.type_name(ty),
                    path: self.path(),
                });
            }
            return Ok(SchemaInputValue::Null);
        }

        match ty.inner {
            Definition::Scalar(id) => self.coerce_scalar(id, value),
            Definition::Enum(id) => self.coerce_enum(id, value),
            Definition::InputObject(id) => self.coerce_input_objet(id, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_objet(
        &mut self,
        input_object_id: InputObjectId,
        value: Value,
    ) -> Result<SchemaInputValue, InputValueError> {
        let input_object = &self.graph[input_object_id];
        let Value::Object(fields) = value else {
            return Err(InputValueError::MissingObject {
                name: self.ctx.strings[input_object.name].to_string(),
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
        for (input_field, input_field_id) in self.graph[input_object.input_field_ids]
            .iter()
            .zip(input_object.input_field_ids)
        {
            match fields.binary_search_by_key(&input_field.name, |(id, _)| StringId::from(*id)) {
                Ok(i) => {
                    let value = std::mem::take(&mut fields[i].1).unwrap();
                    self.value_path.push(input_field.name.into());
                    let value = self.coerce_input_value(input_field.ty, value)?;
                    fields_buffer.push((input_field_id, value));
                    self.value_path.pop();
                }
                Err(_) => {
                    if let Some(default_value_id) = input_field.default_value {
                        fields_buffer.push((input_field_id, self.graph.input_values[default_value_id]));
                    } else if input_field.ty.wrapping.is_required() {
                        return Err(InputValueError::UnexpectedNull {
                            expected: self.type_name(input_field.ty),
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
                input_object: self.ctx.strings[input_object.name].to_string(),
                name: self.ctx.strings[StringId::from(id)].to_string(),
                path: self.path(),
            });
        }
        let ids = self.input_values.append_input_object(&mut fields_buffer);
        self.input_fields_buffer_pool.push(fields_buffer);
        Ok(SchemaInputValue::InputObject(ids))
    }

    fn coerce_enum(&mut self, enum_id: EnumId, value: Value) -> Result<SchemaInputValue, InputValueError> {
        let r#enum = &self.graph[enum_id];
        let name = match &value {
            Value::EnumValue(id) => &self.ctx.strings[StringId::from(*id)],
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: self.ctx.strings[r#enum.name].to_string(),
                    actual: value.into(),
                    path: self.path(),
                })
            }
        };

        let value_ids = r#enum.value_ids;
        match self.graph[value_ids].binary_search_by(|enum_value| self.ctx.strings[enum_value.name].as_str().cmp(name))
        {
            Ok(id) => Ok(SchemaInputValue::EnumValue(r#enum.value_ids.get(id).unwrap())),
            Err(_) => Err(InputValueError::UnknownEnumValue {
                r#enum: self.ctx.strings[r#enum.name].to_string(),
                value: name.to_string(),
                path: self.path(),
            }),
        }
    }

    fn coerce_scalar(&mut self, scalar_id: ScalarId, value: Value) -> Result<SchemaInputValue, InputValueError> {
        match self.graph[scalar_id].ty {
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
                        expected: self.ctx.strings[self.graph[scalar_id].name].to_string(),
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
                            self.input_values[id] = (name.into(), self.coerce_scalar(scalar_id, value)?);
                        }
                        SchemaInputValue::Map(ids)
                    }
                    Value::List(list) => {
                        let ids = self.input_values.reserve_list(list.len());
                        for (value, id) in list.into_vec().into_iter().zip(ids) {
                            let value = self.coerce_scalar(scalar_id, value)?;
                            self.input_values[id] = value;
                        }
                        SchemaInputValue::List(ids)
                    }
                })
            }
        }
        .ok_or_else(|| InputValueError::IncorrectScalarType {
            actual: value.into(),
            expected: self.ctx.strings[self.graph[scalar_id].name].to_string(),
            path: self.path(),
        })
    }

    fn type_name(&self, ty: Type) -> String {
        let mut s = String::new();
        for _ in 0..ty.wrapping.list_wrappings().len() {
            s.push('[');
        }
        s.push_str(match ty.inner {
            Definition::Scalar(id) => &self.ctx.strings[self.graph[id].name],
            Definition::Object(id) => &self.ctx.strings[self.graph[id].name],
            Definition::Interface(id) => &self.ctx.strings[self.graph[id].name],
            Definition::Union(id) => &self.ctx.strings[self.graph[id].name],
            Definition::Enum(id) => &self.ctx.strings[self.graph[id].name],
            Definition::InputObject(id) => &self.ctx.strings[self.graph[id].name],
        });
        if ty.wrapping.inner_is_required() {
            s.push('!');
        }
        for wrapping in ty.wrapping.list_wrappings() {
            s.push(']');
            if wrapping == ListWrapping::RequiredList {
                s.push('!');
            }
        }
        s
    }

    fn path(&self) -> String {
        value_path_to_string(self.ctx, &self.value_path)
    }
}
