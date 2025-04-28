use id_newtypes::IdRange;
use itertools::Itertools;
use schema::{
    EnumDefinition, InputObjectDefinition, InputValueDefinitionId, ListWrapping, MutableWrapping, ScalarDefinition,
    ScalarType, Schema, Type, TypeDefinition, TypeRecord,
};
use serde_json::Value;
use walker::Walk;

use crate::{Location, VariableDefinition, VariableInputValueId, VariableInputValueRecord, VariableInputValues};

use super::{
    error::InputValueError,
    path::{ValuePathSegment, value_path_to_string},
};

pub fn coerce_variable(
    schema: &Schema,
    input_values: &mut VariableInputValues,
    definition: VariableDefinition<'_>,
    value: Value,
) -> Result<VariableInputValueId, InputValueError> {
    let mut ctx = VariableCoercionContext {
        schema,
        input_values,
        location: definition.name_location,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
    };
    let value = ctx.coerce_input_value(definition.ty_record.walk(schema), value)?;
    Ok(input_values.push_value(value))
}

struct VariableCoercionContext<'a> {
    schema: &'a Schema,
    input_values: &'a mut VariableInputValues,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, VariableInputValueRecord)>>,
}

impl<'a> VariableCoercionContext<'a> {
    fn coerce_input_value(&mut self, ty: Type<'a>, value: Value) -> Result<VariableInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_array() && !value.is_null() {
            let mut value = self.coerce_named_type(ty.definition(), value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = VariableInputValueRecord::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type(ty.definition(), ty.wrapping.into(), value)
    }

    fn coerce_type(
        &mut self,
        definition: TypeDefinition<'a>,
        mut wrapping: MutableWrapping,
        value: Value,
    ) -> Result<VariableInputValueRecord, InputValueError> {
        let Some(list_wrapping) = wrapping.pop_outermost_list_wrapping() else {
            if value.is_null() {
                if wrapping.is_required() {
                    return Err(InputValueError::UnexpectedNull {
                        expected: format!("{}!", definition.name()),
                        path: self.path(),
                        location: self.location,
                    });
                }
                return Ok(VariableInputValueRecord::Null);
            }
            return self.coerce_named_type(definition, value);
        };

        match (value, list_wrapping) {
            (Value::Null, ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
                expected: TypeRecord {
                    definition_id: definition.id(),
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }
                .walk(self.schema)
                .to_string(),
                path: self.path(),
                location: self.location,
            }),
            (Value::Null, ListWrapping::NullableList) => Ok(VariableInputValueRecord::Null),
            (Value::Array(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_type(definition, wrapping.clone(), value)?;
                    self.value_path.pop();
                }
                Ok(VariableInputValueRecord::List(ids))
            }
            (value, _) => Err(InputValueError::MissingList {
                actual: value.into(),
                expected: TypeRecord {
                    definition_id: definition.id(),
                    wrapping: {
                        wrapping.push_outermost_list_wrapping(list_wrapping);
                        wrapping.into()
                    },
                }
                .walk(self.schema)
                .to_string(),
                path: self.path(),
                location: self.location,
            }),
        }
    }

    fn coerce_named_type(
        &mut self,
        definition: TypeDefinition<'a>,
        value: Value,
    ) -> Result<VariableInputValueRecord, InputValueError> {
        // At this point the definition should be accessible, otherwise the input value should have
        // been rejected earlier.
        match definition {
            TypeDefinition::Scalar(scalar) => self.coerce_scalar(scalar, value),
            TypeDefinition::Enum(r#enum) => self.coerce_enum(r#enum, value),
            TypeDefinition::InputObject(input_object) => self.coerce_input_objet(input_object, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_objet(
        &mut self,
        input_object: InputObjectDefinition<'a>,
        value: Value,
    ) -> Result<VariableInputValueRecord, InputValueError> {
        let Value::Object(mut fields) = value else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: value.into(),
                path: self.path(),
                location: self.location,
            });
        };

        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();

        if input_object.is_one_of {
            if fields.len() != 1 {
                return Err(InputValueError::ExactlyOneFIeldMustBePresentForOneOfInputObjects {
                    name: input_object.name().to_string(),
                    path: self.path(),
                    message: if fields.is_empty() {
                        "No field was provided".to_string()
                    } else {
                        format!(
                            "{} fields ({}) were provided",
                            fields.len(),
                            fields
                                .iter()
                                .format_with(",", |(name, _), f| f(&format_args!("{name}")))
                        )
                    },
                    location: self.location,
                });
            }
            let name = fields.keys().next().unwrap();
            if let Some(input_field) = input_object
                .input_fields()
                .find(|input_field| !input_field.is_inaccessible() && input_field.name() == name)
            {
                let value = fields.swap_remove(input_field.name()).unwrap();
                self.value_path.push(input_field.as_ref().name_id.into());
                let value = self.coerce_input_value(input_field.ty(), value)?;
                fields_buffer.push((input_field.id, value));
                self.value_path.pop();
            }
        } else {
            for input_field in input_object.input_fields() {
                if input_field.is_inaccessible() {
                    continue;
                }
                match fields.swap_remove(input_field.name()) {
                    None => {
                        if let Some(default_value_id) = input_field.as_ref().default_value_id {
                            fields_buffer
                                .push((input_field.id, VariableInputValueRecord::DefaultValue(default_value_id)));
                        } else if input_field.ty().wrapping.is_required() {
                            self.value_path.push(input_field.as_ref().name_id.into());
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
        Ok(VariableInputValueRecord::InputObject(ids))
    }

    fn coerce_enum(
        &mut self,
        r#enum: EnumDefinition<'a>,
        value: Value,
    ) -> Result<VariableInputValueRecord, InputValueError> {
        let name = match &value {
            Value::String(value) => value.as_str(),
            value => {
                return Err(InputValueError::IncorrectEnumValueType {
                    r#enum: r#enum.name().to_owned(),
                    actual: value.into(),
                    path: self.path(),
                    location: self.location,
                });
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

        Ok(VariableInputValueRecord::EnumValue(value.id))
    }

    fn coerce_scalar(
        &mut self,
        scalar: ScalarDefinition<'a>,
        value: Value,
    ) -> Result<VariableInputValueRecord, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::Unknown) => Ok(match value {
                Value::Null => VariableInputValueRecord::Null,
                Value::Number(n) => {
                    if let Some(n) = n.as_f64() {
                        VariableInputValueRecord::Float(n)
                    } else if let Some(n) = n.as_i64() {
                        VariableInputValueRecord::I64(n)
                    } else {
                        VariableInputValueRecord::U64(n.as_u64().unwrap())
                    }
                }
                Value::String(s) => VariableInputValueRecord::String(s),
                Value::Bool(b) => VariableInputValueRecord::Boolean(b),
                Value::Array(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        let value = self.coerce_scalar(scalar, value)?;
                        self.input_values[id] = value;
                    }
                    VariableInputValueRecord::List(ids)
                }
                Value::Object(fields) => {
                    let ids = self.input_values.reserve_map(fields.len());
                    for ((name, value), id) in fields.into_iter().zip(ids) {
                        self.input_values[id] = (name, self.coerce_scalar(scalar, value)?);
                    }
                    VariableInputValueRecord::Map(ids)
                }
            }),
            (Value::Number(number), ScalarType::Int)
                if number.is_f64() && can_coerce_to_int(number.as_f64().unwrap()) =>
            {
                Ok(VariableInputValueRecord::Int(number.as_f64().unwrap() as i32))
            }
            (Value::Number(number), ScalarType::Int) => {
                let Some(value) = number.as_i64().and_then(|n| i32::try_from(n).ok()) else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(VariableInputValueRecord::Int(value))
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
                Ok(VariableInputValueRecord::Float(value))
            }
            (Value::String(value), ScalarType::String) => Ok(VariableInputValueRecord::String(value)),
            (Value::Bool(value), ScalarType::Boolean) => Ok(VariableInputValueRecord::Boolean(value)),
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

fn can_coerce_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
