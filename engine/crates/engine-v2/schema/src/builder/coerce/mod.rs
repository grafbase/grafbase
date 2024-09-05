mod error;
mod path;

use crate::{
    EnumDefinitionId, Graph, InputObjectDefinitionId, InputValueDefinitionId, ScalarDefinitionId, ScalarType,
    SchemaInputValueId, SchemaInputValueRecord, SchemaInputValues, StringId, TypeRecord,
};
pub use error::*;
use federated_graph::Value;
use id_newtypes::IdRange;
use path::*;
use wrapping::ListWrapping;

use super::{BuildContext, DefinitionId};

pub(super) struct InputValueCoercer<'a> {
    ctx: &'a BuildContext,
    graph: &'a Graph,
    input_values: &'a mut SchemaInputValues,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, SchemaInputValueRecord)>>,
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

    pub fn coerce(&mut self, ty: TypeRecord, value: Value) -> Result<SchemaInputValueId, InputValueError> {
        let value = self.coerce_input_value(ty, value)?;
        Ok(self.input_values.push_value(value))
    }

    fn coerce_input_value(&mut self, ty: TypeRecord, value: Value) -> Result<SchemaInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type(ty, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = SchemaInputValueRecord::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_list(ty, value)
    }

    fn coerce_list(&mut self, mut ty: TypeRecord, value: Value) -> Result<SchemaInputValueRecord, InputValueError> {
        let Some(list_wrapping) = ty.wrapping.pop_list_wrapping() else {
            return self.coerce_named_type(ty, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.type_name(ty.wrapped_by(list_wrapping)),
                path: self.path(),
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(SchemaInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_vec().into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_list(ty, value)?;
                    self.value_path.pop();
                }
                Ok(SchemaInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.type_name(ty.wrapped_by(list_wrapping)),
                path: self.path(),
            }),
        }
    }

    fn coerce_named_type(&mut self, ty: TypeRecord, value: Value) -> Result<SchemaInputValueRecord, InputValueError> {
        if value.is_null() {
            if ty.wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: self.type_name(ty),
                    path: self.path(),
                });
            }
            return Ok(SchemaInputValueRecord::Null);
        }

        match ty.definition_id {
            DefinitionId::Scalar(id) => self.coerce_scalar(id, value),
            DefinitionId::Enum(id) => self.coerce_enum(id, value),
            DefinitionId::InputObject(id) => self.coerce_input_objet(id, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_objet(
        &mut self,
        input_object_id: InputObjectDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let input_object = &self.graph[input_object_id];
        let Value::Object(fields) = value else {
            return Err(InputValueError::MissingObject {
                name: self.ctx.strings[input_object.name_id].to_string(),
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
            match fields.binary_search_by_key(&input_field.name_id, |(id, _)| StringId::from(*id)) {
                Ok(i) => {
                    let value = std::mem::take(&mut fields[i].1).unwrap();
                    self.value_path.push(input_field.name_id.into());
                    let value = self.coerce_input_value(input_field.ty, value)?;
                    fields_buffer.push((input_field_id, value));
                    self.value_path.pop();
                }
                Err(_) => {
                    if let Some(default_value_id) = input_field.default_value_id {
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
                input_object: self.ctx.strings[input_object.name_id].to_string(),
                name: self.ctx.strings[StringId::from(id)].to_string(),
                path: self.path(),
            });
        }
        fields_buffer.sort_unstable_by_key(|(id, _)| *id);
        let ids = self.input_values.append_input_object(&mut fields_buffer);
        self.input_fields_buffer_pool.push(fields_buffer);
        Ok(SchemaInputValueRecord::InputObject(ids))
    }

    fn coerce_enum(
        &mut self,
        enum_id: EnumDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let r#enum = &self.graph[enum_id];
        match &value {
            Value::EnumValue(id) => Ok(SchemaInputValueRecord::EnumValue(crate::EnumValueId::from(id.0))),
            value => Err(InputValueError::IncorrectEnumValueType {
                r#enum: self.ctx.strings[r#enum.name_id].to_string(),
                actual: value.into(),
                path: self.path(),
            }),
        }
    }

    fn coerce_scalar(
        &mut self,
        scalar_id: ScalarDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match self.graph[scalar_id].ty {
            ScalarType::String => match value {
                Value::String(id) => Some(id.into()),
                _ => None,
            }
            .map(SchemaInputValueRecord::String),
            ScalarType::Float => match value {
                Value::Int(n) => Some(n as f64),
                Value::Float(f) => Some(f),
                _ => None,
            }
            .map(SchemaInputValueRecord::Float),
            ScalarType::Int => match value {
                Value::Int(n) => {
                    let n = i32::try_from(n).map_err(|_| InputValueError::IncorrectScalarValue {
                        actual: n.to_string(),
                        expected: self.ctx.strings[self.graph[scalar_id].name_id].to_string(),
                        path: self.path(),
                    })?;
                    Some(n)
                }
                _ => None,
            }
            .map(SchemaInputValueRecord::Int),
            ScalarType::BigInt => match value {
                Value::Int(n) => Some(n),
                _ => None,
            }
            .map(SchemaInputValueRecord::BigInt),
            ScalarType::Boolean => match value {
                Value::Boolean(b) => Some(b),
                _ => None,
            }
            .map(SchemaInputValueRecord::Boolean),
            ScalarType::JSON => {
                return Ok(self
                    .input_values
                    .ingest_arbitrary_federated_value(self.ctx, value)
                    .map_err(|_: super::input_values::InaccessibleEnumValue| {
                        InputValueError::InaccessibleEnumValue { path: self.path() }
                    }))?
            }
        }
        .ok_or_else(|| InputValueError::IncorrectScalarType {
            actual: value.into(),
            expected: self.ctx.strings[self.graph[scalar_id].name_id].to_string(),
            path: self.path(),
        })
    }

    fn type_name(&self, ty: TypeRecord) -> String {
        let mut s = String::new();
        for _ in 0..ty.wrapping.list_wrappings().len() {
            s.push('[');
        }
        s.push_str(match ty.definition_id {
            DefinitionId::Scalar(id) => &self.ctx.strings[self.graph[id].name_id],
            DefinitionId::Object(id) => &self.ctx.strings[self.graph[id].name_id],
            DefinitionId::Interface(id) => &self.ctx.strings[self.graph[id].name_id],
            DefinitionId::Union(id) => &self.ctx.strings[self.graph[id].name_id],
            DefinitionId::Enum(id) => &self.ctx.strings[self.graph[id].name_id],
            DefinitionId::InputObject(id) => &self.ctx.strings[self.graph[id].name_id],
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
