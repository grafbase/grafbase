use cynic_parser::{ConstValue, Value};
use id_newtypes::IdRange;
use itertools::Itertools as _;
use schema::{
    EnumDefinition, InputObjectDefinition, InputValueDefinitionId, ListWrapping, MutableWrapping, ScalarDefinition,
    ScalarType, Type, TypeDefinition, TypeDefinitionId, TypeRecord, Wrapping,
};
use walker::Walk;

use super::{
    super::OperationBinder,
    error::InputValueError,
    path::{ValuePathSegment, value_path_to_string},
};
use crate::{Location, OneOfInputFieldRecord, QueryInputValueId, QueryInputValueRecord, QueryInputValues};

pub fn coerce_variable_default_value<'schema>(
    binder: &mut OperationBinder<'schema, '_>,
    ty: Type<'schema>,
    value: ConstValue<'_>,
) -> QueryInputValueId {
    let mut ctx = QueryValueCoercionContext {
        location: binder.parsed_operation.span_to_location(value.span()),
        binder,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
        is_variable_default_value: true,
    };
    match ctx.coerce_input_value(ty, value.into()) {
        Ok(value) => ctx.query_input_values.push_value(value),
        Err(err) => {
            ctx.binder.errors.push(err.into());
            ctx.query_input_values.push_value(QueryInputValueRecord::Null)
        }
    }
}

pub fn coerce_query_value<'schema>(
    binder: &mut OperationBinder<'schema, '_>,
    ty: Type<'schema>,
    value: cynic_parser::Value<'_>,
) -> QueryInputValueId {
    let mut ctx = QueryValueCoercionContext {
        location: binder.parsed_operation.span_to_location(value.span()),
        binder,
        value_path: Vec::new(),
        input_fields_buffer_pool: Vec::new(),
        is_variable_default_value: false,
    };
    match ctx.coerce_input_value(ty, value) {
        Ok(value) => ctx.query_input_values.push_value(value),
        Err(err) => {
            ctx.binder.errors.push(err.into());
            ctx.query_input_values.push_value(QueryInputValueRecord::Null)
        }
    }
}

struct QueryValueCoercionContext<'binder, 'schema, 'parsed> {
    binder: &'binder mut OperationBinder<'schema, 'parsed>,
    location: Location,
    value_path: Vec<ValuePathSegment>,
    input_fields_buffer_pool: Vec<Vec<(InputValueDefinitionId, QueryInputValueRecord)>>,
    is_variable_default_value: bool,
}

impl QueryValueCoercionContext<'_, '_, '_> {
    fn input_values(&mut self) -> &mut QueryInputValues {
        &mut self.binder.query_input_values
    }
}

impl<'schema, 'parsed> std::ops::Deref for QueryValueCoercionContext<'_, 'schema, 'parsed> {
    type Target = OperationBinder<'schema, 'parsed>;

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
        if self.is_variable_default_value {
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
        if !are_type_compatibles(variable_ty.walk(self.schema), ty) {
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
                value = QueryInputValueRecord::List(IdRange::from_single(self.input_values().push_value(value)));
            }
            return Ok(value);
        }

        self.coerce_type(ty.definition(), ty.wrapping.into(), value)
    }

    fn coerce_type(
        &mut self,
        definition: TypeDefinition<'schema>,
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
            (Value::Null(_), ListWrapping::ListNonNull) => Err(InputValueError::UnexpectedNull {
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
            (Value::Null(_), ListWrapping::List) => Ok(QueryInputValueRecord::Null),
            (Value::List(array), _) => {
                let ids = self.input_values().reserve_list(array.len());
                for ((idx, value), id) in array.into_iter().enumerate().zip(ids) {
                    self.value_path.push(idx.into());
                    self.input_values()[id] = self.coerce_type(definition, wrapping.clone(), value)?;
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
        definition: TypeDefinition<'schema>,
        value: cynic_parser::Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        // At this point the definition should be accessible, otherwise the input value should have
        // been rejected earlier.
        match definition {
            TypeDefinition::Scalar(scalar) => self.coerce_scalar(scalar, value),
            TypeDefinition::Enum(r#enum) => self.coerce_enum(r#enum, value),
            TypeDefinition::InputObject(input_object) => self.coerce_input_object(input_object, value),
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
        let mut fields = object.fields().collect::<Vec<_>>();

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
                                .format_with(",", |field, f| f(&format_args!("{}", field.name())))
                        )
                    },
                    location: self.location,
                });
            }
            let name = fields[0].name();
            if let Some(input_field) = input_object
                .input_fields()
                .find(|input_field| !input_field.is_inaccessible() && input_field.name() == name)
            {
                let field = fields.pop().unwrap();
                self.value_path.push(input_field.as_ref().name_id.into());
                let value = if let Value::Variable(variable) = field.value() {
                    let value = self.variable_ref(variable.name(), input_field.ty())?;
                    let QueryInputValueRecord::Variable(id) = value else {
                        unreachable!()
                    };
                    self.variable_definitions[usize::from(id)].one_of_input_field_usage_record =
                        Some(OneOfInputFieldRecord {
                            object_id: input_object.id,
                            field_id: input_field.id,
                            location: self.parsed_operation.span_to_location(field.value().span()),
                        });
                    value
                } else {
                    self.coerce_input_value(input_field.ty(), field.value())?
                };
                fields_buffer.push((input_field.id, value));
                self.value_path.pop();
            }
        } else {
            for input_field in input_object.input_fields() {
                if input_field.is_inaccessible() {
                    continue;
                }
                if let Some(index) = fields.iter().position(|argument| argument.name() == input_field.name()) {
                    let field = fields.swap_remove(index);
                    self.value_path.push(input_field.as_ref().name_id.into());
                    let value = self.coerce_input_value(input_field.ty(), field.value())?;
                    fields_buffer.push((input_field.id, value));
                    self.value_path.pop();
                } else if let Some(default_value_id) = input_field.as_ref().default_value_id {
                    fields_buffer.push((input_field.id, QueryInputValueRecord::DefaultValue(default_value_id)));
                } else if input_field.ty().wrapping.is_non_null() {
                    self.value_path.push(input_field.as_ref().name_id.into());
                    return Err(InputValueError::UnexpectedNull {
                        expected: input_field.ty().to_string(),
                        path: self.path(),
                        location: self.location,
                    });
                }
            }
        }

        if let Some(field) = fields.first() {
            return Err(InputValueError::UnknownInputField {
                input_object: input_object.name().to_string(),
                name: field.name().to_string(),
                path: self.path(),
                location: self.location,
            });
        }

        // We iterate over input fields in order which is a range, so it should be sorted by the
        // id.
        debug_assert!(fields_buffer.is_sorted_by_key(|(id, _)| *id));
        let ids = self.input_values().append_input_object(&mut fields_buffer);
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

        Ok(QueryInputValueRecord::EnumValue(value.id))
    }

    fn coerce_scalar(
        &mut self,
        scalar: ScalarDefinition<'schema>,
        value: Value<'_>,
    ) -> Result<QueryInputValueRecord, InputValueError> {
        match (value, scalar.as_ref().ty) {
            (value, ScalarType::Unknown) => Ok(match value {
                Value::Null(_) => QueryInputValueRecord::Null,
                Value::Float(n) => QueryInputValueRecord::Float(n.value()),
                Value::Int(i) => QueryInputValueRecord::I64(i.as_i64()),
                Value::String(s) => QueryInputValueRecord::String(s.as_str().into()),
                Value::Boolean(b) => QueryInputValueRecord::Boolean(b.value()),
                Value::List(array) => {
                    let ids = self.input_values().reserve_list(array.len());
                    for (value, id) in array.into_iter().zip(ids) {
                        self.input_values()[id] = self.coerce_scalar(scalar, value)?;
                    }
                    QueryInputValueRecord::List(ids)
                }
                Value::Object(fields) => {
                    let ids = self.input_values().reserve_map(fields.len());
                    for (field, id) in fields.into_iter().zip(ids) {
                        let key = field.name().to_string();
                        self.input_values()[id] = (key, self.coerce_scalar(scalar, field.value())?);
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
            (Value::Int(number), ScalarType::Float) => Ok(QueryInputValueRecord::Float(number.value() as f64)),
            (Value::Float(number), ScalarType::Float) => Ok(QueryInputValueRecord::Float(number.as_f64())),
            (Value::Float(number), ScalarType::Int) if can_coerce_to_int(number.as_f64()) => {
                Ok(QueryInputValueRecord::Int(number.as_f64() as i32))
            }
            (Value::String(value), ScalarType::String) => Ok(QueryInputValueRecord::String(value.as_str().into())),
            (Value::Boolean(value), ScalarType::Boolean) => Ok(QueryInputValueRecord::Boolean(value.value())),
            (Value::Variable(variable), _) => self.variable_ref(
                variable.name(),
                TypeRecord {
                    definition_id: TypeDefinitionId::Scalar(scalar.id),
                    wrapping: Wrapping::default(),
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
fn are_type_compatibles(ty: Type<'_>, used_as: Type<'_>) -> bool {
    (ty.definition_id == used_as.definition_id
        || ty
            .definition()
            .as_composite_type()
            .zip(used_as.definition().as_composite_type())
            .map(|(def, used)| def.is_subset_of(used))
            .unwrap_or_default())
        && (used_as.wrapping.is_equal_or_more_lenient_than(ty.wrapping)
                // if not a list, the current type can be coerced into the proper list wrapping.
            || (
                !ty.wrapping.is_list() && (used_as.wrapping.is_nullable() || ty.wrapping.is_non_null())
            ))
}

fn can_coerce_to_int(float: f64) -> bool {
    float.floor() == float && float < (i32::MAX as f64)
}
