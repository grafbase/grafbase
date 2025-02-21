use crate::{
    DefinitionId, EnumDefinitionId, InputObjectDefinitionId, InputValueDefinitionId, ScalarDefinitionId, ScalarType,
    SchemaInputValueId, SchemaInputValueRecord, TypeRecord, builder::GraphContext,
};
use cynic_parser::ConstValue;
use federated_graph::Value;
use id_newtypes::IdRange;
use wrapping::{ListWrapping, MutableWrapping};

use super::{InputValueError, can_coerce_to_int, value_path_to_string};

impl GraphContext<'_> {
    pub fn coerce_cynic_value(
        &mut self,
        id: InputValueDefinitionId,
        value: ConstValue,
    ) -> Result<SchemaInputValueId, InputValueError> {
        self.value_path.clear();
        self.value_path.push(self.graph[id].name_id.into());
        let value = self.coerce_input_cynic_value(self.graph[id].ty_record, value)?;
        Ok(self.graph.input_values.push_value(value))
    }

    pub fn coerce_fed_value(
        &mut self,
        id: InputValueDefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueId, InputValueError> {
        self.value_path.clear();
        self.value_path.push(self.graph[id].name_id.into());
        let value = self.coerce_input_fed_value(self.graph[id].ty_record, value)?;
        Ok(self.graph.input_values.push_value(value))
    }

    fn coerce_input_cynic_value(
        &mut self,
        ty: TypeRecord,
        value: ConstValue,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type_cynic_value(ty.definition_id, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = SchemaInputValueRecord::List(IdRange::from_single(self.graph.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type_cynic_value(ty.definition_id, ty.wrapping.into(), value)
    }

    fn coerce_input_fed_value(
        &mut self,
        ty: TypeRecord,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
            let mut value = self.coerce_named_type_fed_value(ty.definition_id, value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = SchemaInputValueRecord::List(IdRange::from_single(self.graph.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type_fed_value(ty.definition_id, ty.wrapping.into(), value)
    }

    fn coerce_type_cynic_value(
        &mut self,
        definition_id: DefinitionId,
        mut wrapping: MutableWrapping,
        value: ConstValue,
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
            return self.coerce_named_type_cynic_value(definition_id, value);
        };

        match (value, list_wrapping) {
            (ConstValue::Null(_), ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }),
                path: self.path(),
            }),
            (ConstValue::Null(_), ListWrapping::NullableList) => Ok(SchemaInputValueRecord::Null),
            (ConstValue::List(array), _) => {
                let ids = self.graph.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.graph.input_values[id] =
                        self.coerce_type_cynic_value(definition_id, wrapping.clone(), value)?;
                    self.value_path.pop();
                }
                Ok(SchemaInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }),
                path: self.path(),
            }),
        }
    }

    fn coerce_type_fed_value(
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
            return self.coerce_named_type_fed_value(definition_id, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }),
                path: self.path(),
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(SchemaInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.graph.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_vec().into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.graph.input_values[id] = self.coerce_type_fed_value(definition_id, wrapping.clone(), value)?;
                    self.value_path.pop();
                }
                Ok(SchemaInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: self.type_name(TypeRecord {
                    definition_id,
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }),
                path: self.path(),
            }),
        }
    }

    fn coerce_named_type_cynic_value(
        &mut self,
        definition_id: DefinitionId,
        value: ConstValue,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match definition_id {
            DefinitionId::Scalar(id) => self.coerce_scalar_cynic_value(id, value),
            DefinitionId::Enum(id) => self.coerce_enum_cynic_value(id, value),
            DefinitionId::InputObject(id) => self.coerce_input_object_cynic_value(id, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_named_type_fed_value(
        &mut self,
        definition_id: DefinitionId,
        value: Value,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match definition_id {
            DefinitionId::Scalar(id) => self.coerce_scalar_fed_value(id, value),
            DefinitionId::Enum(id) => self.coerce_enum_fed_value(id, value),
            DefinitionId::InputObject(id) => self.coerce_input_object_fed_value(id, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_object_cynic_value(
        &mut self,
        input_object_id: InputObjectDefinitionId,
        obj: ConstValue,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let input_object = &self.graph[input_object_id];
        let ConstValue::Object(obj) = obj else {
            return Err(InputValueError::MissingObject {
                name: self.ctx.strings[input_object.name_id].to_string(),
                actual: obj.into(),
                path: self.path(),
            });
        };

        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();
        let mut fields = obj.fields().collect::<Vec<_>>();

        for input_field_id in input_object.input_field_ids {
            let input_field = &self.graph[input_field_id];
            let name_id = input_field.name_id;
            let ty_record = input_field.ty_record;
            let default_value_id = input_field.default_value_id;

            if let Some(index) = fields
                .iter()
                .position(|field| field.name() == self.ctx.strings[name_id])
            {
                let field = fields.swap_remove(index);
                self.value_path.push(input_field.name_id.into());
                let value = self.coerce_input_cynic_value(ty_record, field.value())?;
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

        if let Some(field) = fields.first() {
            return Err(InputValueError::UnknownInputField {
                input_object: self.ctx.strings[self.graph[input_object_id].name_id].to_string(),
                name: field.name().to_string(),
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

    fn coerce_input_object_fed_value(
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
                let value = self.coerce_input_fed_value(ty_record, value)?;
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

    fn coerce_enum_cynic_value(
        &mut self,
        enum_id: EnumDefinitionId,
        value: ConstValue,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        let r#enum = &self.graph[enum_id];
        let value = match value {
            ConstValue::Enum(e) => e.as_str(),
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: self.ctx.strings[r#enum.name_id].to_string(),
                    actual: value.into(),
                    path: self.path(),
                });
            }
        };

        for id in r#enum.value_ids {
            if self.ctx.strings[self.graph[id].name_id] == value {
                return Ok(SchemaInputValueRecord::EnumValue(id));
            }
        }
        Err(InputValueError::UnknownEnumValue {
            r#enum: self.ctx.strings[r#enum.name_id].to_string(),
            value: value.to_string(),
            path: self.path(),
        })
    }

    fn coerce_enum_fed_value(
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

    fn coerce_scalar_cynic_value(
        &mut self,
        scalar_id: ScalarDefinitionId,
        value: ConstValue,
    ) -> Result<SchemaInputValueRecord, InputValueError> {
        match self.graph[scalar_id].ty {
            ScalarType::String => match value {
                ConstValue::String(s) => Some(self.ctx.strings.get_or_new(s.value())),
                _ => None,
            }
            .map(SchemaInputValueRecord::String),
            ScalarType::Float => match value {
                ConstValue::Int(n) => Some(n.value() as f64),
                ConstValue::Float(f) => Some(f.value()),
                _ => None,
            }
            .map(SchemaInputValueRecord::Float),
            ScalarType::Int => match value {
                ConstValue::Int(n) => {
                    let n = i32::try_from(n.value()).map_err(|_| InputValueError::IncorrectScalarValue {
                        actual: n.to_string(),
                        expected: self.ctx.strings[self.graph[scalar_id].name_id].to_string(),
                        path: self.path(),
                    })?;
                    Some(n)
                }
                ConstValue::Float(f) if can_coerce_to_int(f.value()) => Some(f.value() as i32),
                _ => None,
            }
            .map(SchemaInputValueRecord::Int),
            ScalarType::BigInt => match value {
                ConstValue::Int(n) => Some(n.value()),
                _ => None,
            }
            .map(SchemaInputValueRecord::BigInt),
            ScalarType::Boolean => match value {
                ConstValue::Boolean(b) => Some(b.value()),
                _ => None,
            }
            .map(SchemaInputValueRecord::Boolean),
            ScalarType::Unknown => {
                return Ok(self.ingest_arbitrary_cynic_value(value));
            }
        }
        .ok_or_else(|| InputValueError::IncorrectScalarType {
            actual: value.into(),
            expected: self.ctx.strings[self.graph[scalar_id].name_id].to_string(),
            path: self.path(),
        })
    }

    fn coerce_scalar_fed_value(
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

    fn ingest_arbitrary_cynic_value(&mut self, value: ConstValue) -> SchemaInputValueRecord {
        match value {
            ConstValue::Null(_) => SchemaInputValueRecord::Null,
            ConstValue::String(s) => SchemaInputValueRecord::String(self.ctx.strings.get_or_new(s.value())),
            ConstValue::Int(n) => SchemaInputValueRecord::BigInt(n.value()),
            ConstValue::Float(f) => SchemaInputValueRecord::Float(f.value()),
            ConstValue::Boolean(b) => SchemaInputValueRecord::Boolean(b.value()),
            ConstValue::Enum(s) => SchemaInputValueRecord::UnboundEnumValue(self.ctx.strings.get_or_new(s.as_str())),
            ConstValue::List(list) => {
                let ids = self.graph.input_values.reserve_list(list.len());
                for (value, id) in list.into_iter().zip(ids) {
                    self.graph.input_values[id] = self.ingest_arbitrary_cynic_value(value);
                }
                SchemaInputValueRecord::List(ids)
            }
            ConstValue::Object(fields) => {
                let ids = self.graph.input_values.reserve_map(fields.len());
                for (field, id) in fields.into_iter().zip(ids) {
                    let name = self.ctx.strings.get_or_new(field.name());
                    let value = self.ingest_arbitrary_cynic_value(field.value());
                    self.graph.input_values[id] = (name, value);
                }
                let ctx = &self.ctx;
                self.graph.input_values[ids].sort_unstable_by(|(left_key, _), (right_key, _)| {
                    ctx.strings.get_by_id(*left_key).cmp(&ctx.strings.get_by_id(*right_key))
                });
                SchemaInputValueRecord::Map(ids)
            }
        }
    }

    fn path(&self) -> String {
        value_path_to_string(self, &self.value_path)
    }
}
