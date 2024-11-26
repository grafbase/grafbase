use std::collections::BTreeMap;

use cynic_parser::{ConstValue, Value};
use id_newtypes::IdRange;
use schema::{
    Definition, DefinitionId, EnumDefinition, InputObjectDefinition, InputValueDefinitionId, ListWrapping,
    MutableWrapping, ScalarDefinition, ScalarType, Type, TypeRecord, Wrapping,
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
    ty: Type<'schema>,
    value: ConstValue<'_>,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        location: binder.parsed_operation.span_to_location(value.span()),
        binder,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
        is_default_value: true,
    };
    let value = ctx.coerce_input_value(ty, value.into())?;
    Ok(ctx.input_values.push_value(value))
}

pub fn coerce_query_value<'schema>(
    binder: &mut Binder<'schema, '_>,
    ty: Type<'schema>,
    value: cynic_parser::Value<'_>,
) -> Result<QueryInputValueId, InputValueError> {
    let mut ctx = QueryValueCoercionContext {
        location: binder.parsed_operation.span_to_location(value.span()),
        binder,
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
    fn variable_ref(&mut self, name: &str, ty: Type<'schema>) -> Result<QueryInputValueRecord, InputValueError> {
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
        value: cynic_parser::Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        if ty.wrapping.is_list() && !value.is_list() && !value.is_null() && !value.is_variable() {
            let mut value = self.coerce_named_type(ty.definition(), value)?;
            for _ in 0..ty.wrapping.list_wrappings().len() {
                value = QueryInputValueRecord::List(IdRange::from_single(self.input_values.push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type(ty.definition(), ty.wrapping.into(), value)
    }

    fn coerce_type(
        &mut self,
        definition: Definition<'schema>,
        mut wrapping: MutableWrapping,
        value: cynic_parser::Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        if let Value::Variable(variable) = value {
            return self.variable_ref(
                variable.name(),
                TypeRecord {
                    definition_id: definition.id(),
                    wrapping: wrapping.into(),
                }
                .walk(self.schema),
            );
        }

        let Some(list_wrapping) = wrapping.pop_outermost_list_wrapping() else {
            if value.is_null() {
                if wrapping.is_required() {
                    return Err(InputValueError::UnexpectedNull {
                        expected: format!("{}!", definition.name()),
                        path: self.path(),
                        location: self.location,
                    });
                }
                return Ok(QueryInputValueRecord::Null);
            }
            return self.coerce_named_type(definition, value);
        };

        match (value, list_wrapping) {
            (Value::Null(_), ListWrapping::RequiredList) => Err(InputValueError::UnexpectedNull {
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
            (Value::Null(_), ListWrapping::NullableList) => Ok(QueryInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.input_values.reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values[id] = self.coerce_type(definition, wrapping.clone(), value)?;
                    self.value_path.pop();
                }
                Ok(QueryInputValueRecord::List(ids))
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
        definition: Definition<'schema>,
        value: cynic_parser::Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        // At this point the definition should be accessible, otherwise the input value should have
        // been rejected earlier.
        match definition {
            Definition::Scalar(scalar) => self.coerce_scalar(scalar, value),
            Definition::Enum(r#enum) => self.coerce_enum(r#enum, value),
            Definition::InputObject(input_object) => self.coerce_input_object(input_object, value),
            _ => unreachable!("Cannot be an output type."),
        }
    }

    fn coerce_input_object(
        &mut self,
        input_object: InputObjectDefinition<'schema>,
        value: cynic_parser::Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        let Value::Object(object) = value else {
            return Err(InputValueError::MissingObject {
                name: input_object.name().to_string(),
                actual: value.into(),
                path: self.path(),
                location: self.location,
            });
        };

        let mut fields_buffer = self.input_fields_buffer_pool.pop().unwrap_or_default();

        let mut fields = object
            .fields()
            .map(|field| (field.name(), field.value()))
            .collect::<BTreeMap<_, _>>();

        for input_field in input_object.input_fields() {
            if input_field.is_inaccessible() {
                continue;
            }
            match fields.remove(input_field.name()) {
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
        value: Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        let name = match value {
            Value::Enum(value) => value.name(),
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
        value: Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::JSON) => Ok(match value {
                Value::Null(_) => QueryInputValueRecord::Null,
                Value::Float(n) => QueryInputValueRecord::Float(n.value()),
                Value::Int(i) => QueryInputValueRecord::BigInt(i.as_i64()),
                Value::String(s) => QueryInputValueRecord::String(s.as_str().into()),
                Value::Boolean(b) => QueryInputValueRecord::Boolean(b.value()),
                Value::List(array) => {
                    let ids = self.input_values.reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        self.input_values[id] = self.coerce_scalar(scalar, value)?;
                    }
                    QueryInputValueRecord::List(ids)
                }
                Value::Object(fields) => {
                    let ids = self.input_values.reserve_map(fields.len());
                    for (field, id) in fields.into_iter().zip(ids) {
                        let key = field.name().to_string();
                        self.input_values[id] = (key, self.coerce_scalar(scalar, field.value())?);
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
            (Value::Int(number), ScalarType::Int) => {
                let Some(value) = i32::try_from(number.as_i64()).ok() else {
                    return Err(InputValueError::IncorrectScalarValue {
                        actual: number.to_string(),
                        expected: scalar.name().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                };
                Ok(QueryInputValueRecord::Int(value))
            }
            (Value::Int(number), ScalarType::BigInt) => Ok(QueryInputValueRecord::BigInt(number.as_i64())),
            (Value::Int(number), ScalarType::Float) => Ok(QueryInputValueRecord::Float(number.value() as f64)),
            (Value::Float(number), ScalarType::Float) => Ok(QueryInputValueRecord::Float(number.as_f64())),
            (Value::Float(number), ScalarType::Int) if can_coerce_to_int(number.as_f64()) => {
                Ok(QueryInputValueRecord::Int(number.as_f64() as i32))
            }
            (Value::Float(number), ScalarType::BigInt) if can_coerce_to_big_int(number.as_f64()) => {
                Ok(QueryInputValueRecord::BigInt(number.as_f64() as i64))
            }
            (Value::String(value), ScalarType::String) => Ok(QueryInputValueRecord::String(value.as_str().into())),
            (Value::Boolean(value), ScalarType::Boolean) => Ok(QueryInputValueRecord::Boolean(value.value())),
            (Value::Variable(variable), _) => self.variable_ref(
                variable.name(),
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

fn can_coerce_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}

fn can_coerce_to_big_int(float: f64) -> bool {
    float.floor() == float && float < (i64::MAX as f64)
}
