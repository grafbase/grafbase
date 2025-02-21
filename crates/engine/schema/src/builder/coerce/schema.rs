use crate::{
    DefinitionId, EnumDefinitionId, InputObjectDefinitionId, InputValueDefinitionId, ScalarDefinitionId, ScalarType,
    SchemaInputValueId, SchemaInputValueRecord, TypeRecord, builder::GraphContext,
};
use federated_graph::Value;
use id_newtypes::IdRange;
use wrapping::{ListWrapping, MutableWrapping};

use super::{InputValueError, can_coerce_to_int, value_path_to_string};

impl GraphContext<'_> {
    pub fn coerce(&mut self, id: InputValueDefinitionId, value: Value) -> Result<SchemaInputValueId, InputValueError> {
        self.value_path.clear();
        self.value_path.push(self.graph[id].name_id.into());
        let value = self.coerce_input_value(self.graph[id].ty_record, value)?;
        Ok(self.graph.input_values.push_value(value))
    }

    fn coerce_input_value(&mut self, ty: TypeRecord, value: Value) -> Result<SchemaInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type(ty.definition_id, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = SchemaInputValueRecord::List(IdRange::from_single(self.graph.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type(ty.definition_id, ty.wrapping.into(), value)
    }

    fn coerce_type(
        &mut self,
        definition_id: DefinitionId,
        mut wrapping: MutableWrapping,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let Some(list_wrapping) = wrapping.pop_outermost_list_wrapping() else {
            if value.is_null() {
                if wrapping.is_required() {
                    return Err(InputValueError::UnexpectedNull {
                        expected: self.type_name(TypeRecord {
                            definition_id,
                            wrapping: wrapping.into(),
                        }),
                        path: self.path(),
                    });
                }
                return Ok(SchemaInputValueRecord::Null);
            }
            return self.coerce_named_type(definition_id, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: wrapping.into(),
                }),
                path: self.path(),
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(SchemaInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.graph.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_vec().into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.graph.input_values[id] = self.coerce_type(definition_id, wrapping.clone(), value)?;
                    self.value_path.pop();
                }
                Ok(SchemaInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: wrapping.into(),
                }),
                path: self.path(),
            }),
        }
    }

    fn coerce_named_type(
        &mut self,
        definition_id: DefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match definition_id {
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

        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();
        let mut fields = Vec::from(fields);
        for input_field_id in input_object.input_field_ids {
            let input_field = &self.graph[input_field_id];
            let name_id = input_field.name_id;
            let ty_record = input_field.ty_record;
            let default_value_id = input_field.default_value_id;

            if let Some(index) = fields
                .iter()
                .position(|(id, _)| self.federated_graph[*id] == self.ctx.strings[name_id])
            {
                let (_, value) = fields.swap_remove(index);
                self.value_path.push(input_field.name_id.into());
                let value = self.coerce_input_value(ty_record, value)?;
                fields_buffer.push((input_field_id, value));
                self.value_path.pop();
            } else if let Some(default_value_id) = default_value_id {
                fields_buffer.push((input_field_id, self.graph.input_values[default_value_id]));
            } else if ty_record.wrapping.is_required() {
                self.value_path.push(name_id.into());
                return Err(InputValueError::UnexpectedNull {
                    expected: self.type_name(ty_record),
                    path: self.path(),
                });
            }
        }

        if let Some((id, _)) = fields.first() {
            return Err(InputValueError::UnknownInputField {
                input_object: self.ctx.strings[self.graph[input_object_id].name_id].to_string(),
                name: self.federated_graph[*id].to_string(),
                path: self.path(),
            });
        }

        // We iterate over input fields in order which is a range, so it should be sorted by the
        // id.
        debug_assert!(fields_buffer.is_sorted_by_key(|(id, _)| *id));
        let ids = self.graph.input_values.append_input_object(&mut fields_buffer);
        self.input_fields_buffer_pool.push(fields_buffer);
        Ok(SchemaInputValueRecord::InputObject(ids))
    }

    fn coerce_enum(
        &mut self,
        enum_id: EnumDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let r#enum = &self.graph[enum_id];
        let value_id = match &value {
            Value::EnumValue(id) => self.federated_graph[*id].value,
            Value::UnboundEnumValue(id) => *id,
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: self.ctx.strings[r#enum.name_id].to_string(),
                    actual: value.into(),
                    path: self.path(),
                });
            }
        };
        let string_value = &self.federated_graph[value_id];
        for id in r#enum.value_ids {
            if &self.ctx.strings[self.graph[id].name_id] == string_value {
                return Ok(SchemaInputValueRecord::EnumValue(id));
            }
        }
        Err(InputValueError::UnknownEnumValue {
            r#enum: self.ctx.strings[r#enum.name_id].to_string(),
            value: string_value.to_string(),
            path: self.path(),
        })
    }

    fn coerce_scalar(
        &mut self,
        scalar_id: ScalarDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match self.graph[scalar_id].ty {
            ScalarType::String => match value {
                Value::String(id) => Some(self.ctx.strings.get_or_new(&self.federated_graph[id])),
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
                Value::Float(f) if can_coerce_to_int(f) => Some(f as i32),
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
            ScalarType::Unknown => {
                return Ok(self.ingest_arbitrary_value(value));
            }
        }
        .ok_or_else(|| InputValueError::IncorrectScalarType {
            actual: value.into(),
            expected: self.ctx.strings[self.graph[scalar_id].name_id].to_string(),
            path: self.path(),
        })
    }

    fn path(&self) -> String {
        value_path_to_string(self, &self.value_path)
    }
}
