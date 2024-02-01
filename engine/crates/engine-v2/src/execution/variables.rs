use std::{collections::HashMap, fmt::Write};

use engine_value::{ConstValue, Name};
use indexmap::IndexMap;
use schema::{DataType, InputObjectId, ListWrapping, Schema, StringId};

use crate::{
    request::{Location, Operation, VariableDefinition, VariableDefinitionId},
    response::GraphqlError,
};

#[derive(Debug, thiserror::Error)]
pub enum VariableError {
    #[error("Missing variable '{name}'")]
    MissingVariable { name: String, location: Location },
    #[error("Variable ${name} got an invalid value: {error}")]
    Coercion {
        name: String,
        error: CoercionError,
        location: Location,
    },
}

impl From<VariableError> for GraphqlError {
    fn from(err: VariableError) -> Self {
        let locations = match err {
            VariableError::MissingVariable { location, .. } => vec![location],
            VariableError::Coercion { location, .. } => vec![location],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            ..Default::default()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoercionError {
    #[error("found a null where we expected a {expected}{path}")]
    UnexpectedNull { expected: String, path: String },
    #[error("found a {actual} value where we expected a {expected}{path}")]
    MissingList {
        actual: ValueKind,
        expected: String,
        path: String,
    },
    #[error("found a {actual} value where we expected a '{name}' input object{path}")]
    MissingObject {
        name: String,
        actual: ValueKind,
        path: String,
    },
    #[error("found a {actual} value where we expected a {expected} scalar{path}")]
    IncorrectScalar {
        actual: ValueKind,
        expected: String,
        path: String,
    },
    #[error("found a {actual} value where we expected a '{name}' enum{path}")]
    IncorrectEnum {
        name: String,
        actual: ValueKind,
        path: String,
    },
    #[error("found the value '{actual}' value where we expected a value of the '{name}' enum{path}")]
    IncorrectEnumValue { name: String, actual: String, path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum ValueKind {
    String,
    Integer,
    Float,
    Object,
    Boolean,
    List,
    Null,
}

impl ValueKind {
    fn of_value(value: &ConstValue) -> Self {
        match value {
            ConstValue::Null => ValueKind::Null,
            ConstValue::Number(number) if number.is_f64() => ValueKind::Float,
            ConstValue::Number(_) => ValueKind::Integer,
            ConstValue::String(_) => ValueKind::String,
            ConstValue::Boolean(_) => ValueKind::Boolean,
            ConstValue::Binary(_) => ValueKind::String,
            ConstValue::Enum(_) => ValueKind::String,
            ConstValue::List(_) => ValueKind::List,
            ConstValue::Object(_) => ValueKind::Object,
        }
    }
}

pub struct Variables {
    inner: HashMap<String, Variable>,
}

pub struct Variable {
    pub value: Option<ConstValue>,
    pub definition_id: VariableDefinitionId,
}

impl Variables {
    pub fn from_request(
        operation: &Operation,
        schema: &Schema,
        variables: &mut engine_value::Variables,
    ) -> Result<Self, Vec<VariableError>> {
        let mut coerced = HashMap::new();
        let mut errors = vec![];

        for (id, definition) in operation.variable_definitions.iter().enumerate() {
            match coerce_variable_value(definition, schema, variables, VariablePath::new(&definition.name)) {
                Ok(value) => {
                    coerced.insert(
                        definition.name.clone(),
                        Variable {
                            value,
                            definition_id: id.into(),
                        },
                    );
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self { inner: coerced })
    }

    pub fn get(&self, name: &str) -> Option<&Variable> {
        self.inner.get(name)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Variable)> {
        self.inner.iter()
    }
}

/// Validates & coerces a variable value
///
/// An implementation of http://spec.graphql.org/October2021/#sec-Coercing-Variable-Values
fn coerce_variable_value(
    variable_definition: &VariableDefinition,
    schema: &Schema,
    variable_values: &mut engine_value::Variables,
    path: VariablePath,
) -> Result<Option<ConstValue>, VariableError> {
    let variable_name = engine_value::Name::new(&variable_definition.name);
    let has_value = variable_values.contains_key(&variable_name);
    if !has_value && variable_definition.default_value.is_some() {
        return Ok(Some(variable_definition.default_value.clone().unwrap()));
    }

    if variable_definition.r#type.wrapping.is_required() {
        if !has_value {
            return Err(VariableError::MissingVariable {
                name: variable_name.to_string(),
                location: variable_definition.name_location,
            });
        }
        if variable_values.get(&variable_name) == Some(&ConstValue::Null) {
            return Err(VariableError::Coercion {
                name: variable_name.to_string(),
                location: variable_definition.name_location,
                error: CoercionError::UnexpectedNull {
                    expected: type_to_string(&variable_definition.r#type, schema),
                    path: path.to_error_string(schema),
                },
            });
        }
    }

    if !has_value {
        return Ok(None);
    }

    if variable_values.get(&variable_name) == Some(&ConstValue::Null) {
        return Ok(Some(ConstValue::Null));
    }

    Ok(Some(
        coerce_value(
            variable_values.remove(&variable_name).unwrap(),
            &variable_definition.r#type,
            schema,
            path,
        )
        .map_err(|error| VariableError::Coercion {
            name: variable_name.to_string(),
            error,
            location: variable_definition.name_location,
        })?,
    ))
}

fn coerce_value(
    mut value: ConstValue,
    r#type: &schema::Type,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    if r#type.wrapping.is_required() && value.is_null() {
        return Err(CoercionError::UnexpectedNull {
            expected: type_to_string(r#type, schema),
            path: path.to_error_string(schema),
        });
    }

    if r#type.wrapping.is_list() && !&value.is_array() && !value.is_null() {
        value = coerce_named_type(value, r#type, schema, path)?;
        for _ in &r#type.wrapping.list_wrapping {
            value = ConstValue::List(vec![value]);
        }
        return Ok(value);
    }

    let lists = r#type.wrapping.list_wrapping.iter().rev().copied().collect::<Vec<_>>();
    coerce_list(&lists, value, r#type, schema, path)
}

fn coerce_list(
    lists: &[ListWrapping],
    value: ConstValue,
    r#type: &schema::Type,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    let Some(expected_nullability) = lists.first() else {
        return coerce_named_type(value, r#type, schema, path);
    };

    match (value, expected_nullability) {
        (ConstValue::Null, ListWrapping::RequiredList) => Err(CoercionError::UnexpectedNull {
            expected: unwrapped_type_to_string(r#type, schema, lists),
            path: path.to_error_string(schema),
        }),
        (ConstValue::Null, ListWrapping::NullableList) => Ok(ConstValue::Null),
        (ConstValue::List(list), _) => Ok(ConstValue::List(
            list.into_iter()
                .enumerate()
                .map(|(index, item)| coerce_list(&lists[1..], item, r#type, schema, path.child(index)))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        (value, _) => Err(CoercionError::MissingList {
            actual: ValueKind::of_value(&value),
            expected: unwrapped_type_to_string(r#type, schema, lists),
            path: path.to_error_string(schema),
        }),
    }
}

fn coerce_named_type(
    value: ConstValue,
    r#type: &schema::Type,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    if value.is_null() {
        return if r#type.wrapping.inner_is_required {
            Err(CoercionError::UnexpectedNull {
                expected: unwrapped_type_to_string(r#type, schema, &[]),
                path: path.to_error_string(schema),
            })
        } else {
            Ok(ConstValue::Null)
        };
    }

    match r#type.inner {
        schema::Definition::Scalar(scalar) => coerce_scalar(value, &schema[scalar], schema, path),
        schema::Definition::Enum(id) => coerce_enum(value, &schema[id], schema, path),
        schema::Definition::InputObject(object) => coerce_input_object(value, object, schema, path),
        schema::Definition::Object(_) | schema::Definition::Interface(_) | schema::Definition::Union(_) => {
            unreachable!("variables can't be output types.")
        }
    }
}

fn coerce_scalar(
    value: ConstValue,
    scalar: &schema::Scalar,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    match (value, scalar.data_type) {
        (ConstValue::Number(number), DataType::Int | DataType::BigInt) if !number.is_f64() => {
            Ok(ConstValue::Number(number))
        }
        (ConstValue::Number(number), DataType::Float) => Ok(ConstValue::Number(number)),
        (ConstValue::String(value), DataType::String) => Ok(ConstValue::String(value)),
        (ConstValue::Boolean(value), DataType::Boolean) => Ok(ConstValue::Boolean(value)),
        (ConstValue::Binary(value), DataType::String) => Ok(ConstValue::Binary(value)),
        (ConstValue::Enum(value), DataType::String) => Ok(ConstValue::Enum(value)),
        (actual, _) => Err(CoercionError::IncorrectScalar {
            actual: ValueKind::of_value(&actual),
            expected: schema[scalar.name].to_string(),
            path: path.to_error_string(schema),
        }),
    }
}

fn coerce_enum(
    value: ConstValue,
    enum_: &schema::Enum,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    let value_str = match &value {
        ConstValue::String(value) => value.as_str(),
        ConstValue::Enum(value) => value.as_str(),
        value => {
            return Err(CoercionError::IncorrectEnum {
                name: schema[enum_.name].to_string(),
                actual: ValueKind::of_value(value),
                path: path.to_error_string(schema),
            })
        }
    };

    if !enum_.values.iter().any(|value| schema[value.name] == value_str) {
        return Err(CoercionError::IncorrectEnumValue {
            name: schema[enum_.name].to_string(),
            actual: value_str.to_string(),
            path: path.to_error_string(schema),
        });
    }

    Ok(value)
}

fn coerce_input_object(
    value: ConstValue,
    object_id: InputObjectId,
    schema: &Schema,
    path: VariablePath,
) -> Result<ConstValue, CoercionError> {
    let ConstValue::Object(mut fields) = value else {
        return Err(CoercionError::MissingObject {
            name: schema[schema[object_id].name].clone(),
            actual: ValueKind::of_value(&value),
            path: path.to_error_string(schema),
        });
    };

    let mut coerced = IndexMap::new();
    for field in schema.walker().walk(object_id).input_fields() {
        match fields.shift_remove(field.name()) {
            None | Some(ConstValue::Null) if field.ty().wrapping().is_required() => {
                return Err(CoercionError::UnexpectedNull {
                    expected: field.ty().to_string(),
                    path: path.to_error_string(schema),
                });
            }
            None => {}
            Some(value) => {
                coerced.insert(
                    Name::new(field.name()),
                    coerce_value(value, field.ty().as_ref(), schema, path.child(field.as_ref().name))?,
                );
            }
        }
    }

    Ok(ConstValue::Object(coerced))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariablePath(String, im::Vector<VariablePathSegment>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum VariablePathSegment {
    Field(StringId),
    Index(usize),
}

impl From<usize> for VariablePathSegment {
    fn from(index: usize) -> VariablePathSegment {
        VariablePathSegment::Index(index)
    }
}

impl From<StringId> for VariablePathSegment {
    fn from(id: StringId) -> VariablePathSegment {
        VariablePathSegment::Field(id)
    }
}

impl VariablePath {
    fn new(variable: &str) -> Self {
        VariablePath(variable.to_string(), Default::default())
    }

    fn child(&self, segment: impl Into<VariablePathSegment>) -> Self {
        let mut child = self.clone();
        child.1.push_back(segment.into());
        child
    }

    fn to_error_string(&self, schema: &Schema) -> String {
        let mut output = String::new();
        write!(&mut output, " at ${}", self.0).ok();
        for segment in self.1.iter() {
            match segment {
                VariablePathSegment::Field(id) => {
                    write!(&mut output, ".{}", schema[*id]).ok();
                }
                VariablePathSegment::Index(idx) => {
                    write!(&mut output, ".{idx}").ok();
                }
            }
        }

        output
    }
}

fn type_to_string(ty: &schema::Type, schema: &Schema) -> String {
    unwrapped_type_to_string(ty, schema, &ty.wrapping.list_wrapping)
}

fn unwrapped_type_to_string(ty: &schema::Type, schema: &Schema, wrapping: &[ListWrapping]) -> String {
    let mut output = String::new();
    for _ in wrapping.iter() {
        write!(&mut output, "[").ok();
    }
    write!(&mut output, "{}", schema.walker().walk(ty.inner).name()).ok();
    if ty.wrapping.inner_is_required {
        write!(&mut output, "!").ok();
    }
    for wrapping in wrapping.iter().rev() {
        write!(&mut output, "]").ok();
        if *wrapping == ListWrapping::RequiredList {
            write!(&mut output, "!").ok();
        }
    }

    output
}
