use crate::{
    EnumDefinitionId, InputObjectDefinitionId, InputValueDefinitionId, ScalarDefinitionId, ScalarType,
    SchemaInputValueId, SchemaInputValueRecord, TypeDefinitionId, TypeRecord,
    builder::{GraphBuilder, sdl::ConstValue},
};
use id_newtypes::IdRange;
use itertools::Itertools as _;
use wrapping::{ListWrapping, MutableWrapping};

use super::{InputValueError, can_coerce_to_int};

impl GraphBuilder<'_> {
    pub(crate) fn coerce_input_value(
        &mut self,
        id: InputValueDefinitionId,
        value: ConstValue,
    ) -> Result<SchemaInputValueId, InputValueError> {
        self.value_path.clear();
        self.value_path.push(self.graph[id].name_id.into());
        let value = coerce_input_value(self, self.graph[id].ty_record, value)?;
        Ok(self.graph.input_values.push_value(value))
    }
}

fn coerce_input_value(
    builder: &mut GraphBuilder,
    ty: TypeRecord,
    value: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    if ty.wrapping.is_list() && !value.is_list() && !value.is_null() {
        let mut value = coerce_named_type_value(builder, ty.definition_id, value)?;
        for _ in 0..ty.wrapping.list_wrappings().len() {
            value = SchemaInputValueRecord::List(IdRange::from_single(builder.graph.input_values.push_value(value)));
        }
        return Ok(value);
    }

    coerce_type_value(builder, ty.definition_id, ty.wrapping.into(), value)
}

fn coerce_type_value(
    builder: &mut GraphBuilder,
    definition_id: TypeDefinitionId,
    mut wrapping: MutableWrapping,
    value: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    let Some(list_wrapping) = wrapping.pop_outermost_list_wrapping() else {
        if value.is_null() {
            if wrapping.is_required() {
                return Err(InputValueError::UnexpectedNull {
                    expected: builder.type_name(TypeRecord {
                        definition_id,
                        wrapping: wrapping.into(),
                    }),
                    path: builder.value_path_string(),
                });
            }
            return Ok(SchemaInputValueRecord::Null);
        }
        return coerce_named_type_value(builder, definition_id, value);
    };

    match (value, list_wrapping) {
        (ConstValue::Null(_), ListWrapping::ListNonNull) => Err(InputValueError::UnexpectedNull {
            expected: builder.type_name(TypeRecord {
                definition_id,
                wrapping: {
                    wrapping.push_outermost_list_wrapping(list_wrapping);
                    wrapping.into()
                },
            }),
            path: builder.value_path_string(),
        }),
        (ConstValue::Null(_), ListWrapping::List) => Ok(SchemaInputValueRecord::Null),
        (ConstValue::List(array), _) => {
            let ids = builder.graph.input_values.reserve_list(array.len());
            for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                builder.value_path.push(idx.into());
                builder.graph.input_values[id] = coerce_type_value(builder, definition_id, wrapping.clone(), value)?;
                builder.value_path.pop();
            }
            Ok(SchemaInputValueRecord::List(ids))
        }
        (value, _) => Err(InputValueError::MissingList {
            actual: value.into(),
            expected: builder.type_name(TypeRecord {
                definition_id,
                wrapping: {
                    wrapping.push_outermost_list_wrapping(list_wrapping);
                    wrapping.into()
                },
            }),
            path: builder.value_path_string(),
        }),
    }
}

fn coerce_named_type_value(
    builder: &mut GraphBuilder,
    definition_id: TypeDefinitionId,
    value: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    match definition_id {
        TypeDefinitionId::Scalar(id) => coerce_scalar_value(builder, id, value),
        TypeDefinitionId::Enum(id) => coerce_enum_value(builder, id, value),
        TypeDefinitionId::InputObject(id) => coerce_input_object_value(builder, id, value),
        _ => unreachable!("Cannot be an output type."),
    }
}

fn coerce_input_object_value(
    builder: &mut GraphBuilder,
    input_object_id: InputObjectDefinitionId,
    obj: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    let input_object = &builder.graph[input_object_id];
    let ConstValue::Object(object) = obj else {
        return Err(InputValueError::MissingObject {
            name: builder[input_object.name_id].clone(),
            actual: obj.into(),
            path: builder.value_path_string(),
        });
    };

    let mut fields_buffer = builder.input_fields_buffer_pool.pop().unwrap_or_default();
    let mut fields = object.fields().collect::<Vec<_>>();

    if input_object.is_one_of {
        if fields.len() != 1 {
            return Err(InputValueError::ExactlyOneFIeldMustBePresentForOneOfInputObjects {
                name: builder[builder.graph[input_object_id].name_id].clone(),
                message: if fields.is_empty() {
                    "No field was provided".to_string()
                } else {
                    format!(
                        "{} fields ({}) were provided",
                        fields.len(),
                        fields
                            .iter()
                            .format_with(",", |field, f| f(&format_args!("{}", field.name())))
                    )
                },
                path: builder.value_path_string(),
            });
        }
        let name = fields[0].name();
        if let Some(id) = input_object
            .input_field_ids
            .into_iter()
            .find(|id| builder[builder.graph[*id].name_id] == name)
        {
            let input_field = &builder.graph[id];
            let field = fields.pop().unwrap();
            builder.value_path.push(input_field.name_id.into());
            let value = coerce_input_value(builder, input_field.ty_record, field.value())?;
            fields_buffer.push((id, value));
            builder.value_path.pop();
        }
    } else {
        for input_field_id in input_object.input_field_ids {
            let input_field = &builder.graph[input_field_id];
            let name_id = input_field.name_id;
            let ty_record = input_field.ty_record;
            let default_value_id = input_field.default_value_id;

            builder.value_path.push(input_field.name_id.into());
            let value = if let Some(index) = fields.iter().position(|field| field.name() == builder[name_id]) {
                let field = fields.swap_remove(index);
                coerce_input_value(builder, ty_record, field.value())?
            } else if let Some(default_value_id) = default_value_id {
                builder.graph.input_values[default_value_id]
            } else if ty_record.wrapping.is_non_null() {
                return Err(InputValueError::UnexpectedNull {
                    expected: builder.type_name(ty_record),
                    path: builder.value_path_string(),
                });
            } else {
                builder.value_path.pop();
                continue;
            };

            fields_buffer.push((input_field_id, value));
            builder.value_path.pop();
        }
    }

    if let Some(field) = fields.first() {
        return Err(InputValueError::UnknownInputField {
            input_object: builder[builder.graph[input_object_id].name_id].clone(),
            name: field.name().to_owned(),
            path: builder.value_path_string(),
        });
    }

    // We iterate over input fields in order which is a range, so it should be sorted by the
    // id.
    debug_assert!(fields_buffer.is_sorted_by_key(|(id, _)| *id));
    let ids = builder.graph.input_values.append_input_object(&mut fields_buffer);
    builder.input_fields_buffer_pool.push(fields_buffer);
    Ok(SchemaInputValueRecord::InputObject(ids))
}

fn coerce_enum_value(
    builder: &mut GraphBuilder,
    enum_id: EnumDefinitionId,
    value: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    let r#enum = &builder.graph[enum_id];
    let value = match value {
        ConstValue::Enum(e) => e.as_str(),
        value => {
            return Err(InputValueError::IncorrectEnumValueType {
                r#enum: builder[r#enum.name_id].clone(),
                actual: value.into(),
                path: builder.value_path_string(),
            });
        }
    };

    for id in r#enum.value_ids {
        if builder[builder.graph[id].name_id] == value {
            return Ok(SchemaInputValueRecord::EnumValue(id));
        }
    }
    Err(InputValueError::UnknownEnumValue {
        r#enum: builder[r#enum.name_id].clone(),
        value: value.to_string(),
        path: builder.value_path_string(),
    })
}

fn coerce_scalar_value(
    builder: &mut GraphBuilder,
    scalar_id: ScalarDefinitionId,
    value: ConstValue,
) -> Result<SchemaInputValueRecord, InputValueError> {
    match builder.graph[scalar_id].ty {
        ScalarType::String => match value {
            ConstValue::String(s) => Some(builder.ingest_str(s.value())),
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
                    expected: builder[builder.graph[scalar_id].name_id].clone(),
                    path: builder.value_path_string(),
                })?;
                Some(n)
            }
            ConstValue::Float(f) if can_coerce_to_int(f.value()) => Some(f.value() as i32),
            _ => None,
        }
        .map(SchemaInputValueRecord::Int),
        ScalarType::Boolean => match value {
            ConstValue::Boolean(b) => Some(b.value()),
            _ => None,
        }
        .map(SchemaInputValueRecord::Boolean),
        ScalarType::Unknown => {
            return Ok(ingest_arbitrary_value(builder, value));
        }
    }
    .ok_or_else(|| InputValueError::IncorrectScalarType {
        actual: value.into(),
        expected: builder[builder.graph[scalar_id].name_id].clone(),
        path: builder.value_path_string(),
    })
}

fn ingest_arbitrary_value(builder: &mut GraphBuilder, value: ConstValue) -> SchemaInputValueRecord {
    match value {
        ConstValue::Null(_) => SchemaInputValueRecord::Null,
        ConstValue::String(s) => SchemaInputValueRecord::String(builder.ingest_str(s.value())),
        ConstValue::Int(n) => SchemaInputValueRecord::I64(n.value()),
        ConstValue::Float(f) => SchemaInputValueRecord::Float(f.value()),
        ConstValue::Boolean(b) => SchemaInputValueRecord::Boolean(b.value()),
        ConstValue::Enum(s) => SchemaInputValueRecord::UnboundEnumValue(builder.ingest_str(s.as_str())),
        ConstValue::List(list) => {
            let ids = builder.graph.input_values.reserve_list(list.len());
            for (value, id) in list.into_iter().zip(ids) {
                builder.graph.input_values[id] = ingest_arbitrary_value(builder, value);
            }
            SchemaInputValueRecord::List(ids)
        }
        ConstValue::Object(fields) => {
            let ids = builder.graph.input_values.reserve_map(fields.len());
            for (field, id) in fields.into_iter().zip(ids) {
                let name = builder.ingest_str(field.name());
                let value = ingest_arbitrary_value(builder, field.value());
                builder.graph.input_values[id] = (name, value);
            }
            let ctx = &builder.ctx;
            builder.graph.input_values[ids]
                .sort_unstable_by(|(left_key, _), (right_key, _)| ctx[*left_key].cmp(&ctx[*right_key]));
            SchemaInputValueRecord::Map(ids)
        }
    }
}
