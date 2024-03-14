use engine_value::{ConstValue, Value};

use crate::request::Location;

#[derive(Debug, thiserror::Error)]
pub enum InputValueError {
    #[error("Found a null where we expected a {expected}{path}")]
    UnexpectedNull {
        expected: String,
        path: String,
        location: Location,
    },
    #[error("Found a {actual} value where we expected a {expected}{path}")]
    MissingList {
        actual: ValueKind,
        expected: String,
        path: String,
        location: Location,
    },
    #[error("Found a {actual} value where we expected a '{name}' input object{path}")]
    MissingObject {
        name: String,
        actual: ValueKind,
        path: String,
        location: Location,
    },
    #[error("Found a {actual} value where we expected a {expected} scalar{path}")]
    IncorrectScalarType {
        actual: ValueKind,
        expected: String,
        path: String,
        location: Location,
    },
    #[error("Found value {actual} which cannot be coerced into a {expected} scalar{path}")]
    IncorrectScalarValue {
        actual: String,
        expected: String,
        path: String,
        location: Location,
    },
    #[error("Found a {actual} value where we expected a {r#enum} enum value{path}")]
    IncorrectEnumValueType {
        r#enum: String,
        actual: ValueKind,
        path: String,
        location: Location,
    },
    #[error("Unknown enum value '{value}' for enum {r#enum}{path}")]
    UnknownEnumValue {
        r#enum: String,
        value: String,
        path: String,
        location: Location,
    },
    #[error(
        "Variable ${name} doesn't have the right type. Declared as '{variable_ty}' but used as '{actual_ty}'{path}"
    )]
    IncorrectVariableType {
        name: String,
        variable_ty: String,
        actual_ty: String,
        location: Location,
        path: String,
    },
    #[error("Input object {input_object} does not have a field named '{name}'{path}")]
    UnknownInputField {
        input_object: String,
        name: String,
        location: Location,
        path: String,
    },
    #[error("Unknown variable ${name}{path}")]
    UnknownVariable {
        name: String,
        location: Location,
        path: String,
    },
}

impl InputValueError {
    pub(crate) fn location(&self) -> Location {
        match self {
            InputValueError::UnexpectedNull { location, .. }
            | InputValueError::MissingList { location, .. }
            | InputValueError::MissingObject { location, .. }
            | InputValueError::IncorrectScalarType { location, .. }
            | InputValueError::IncorrectScalarValue { location, .. }
            | InputValueError::IncorrectEnumValueType { location, .. }
            | InputValueError::UnknownVariable { location, .. }
            | InputValueError::IncorrectVariableType { location, .. }
            | InputValueError::UnknownInputField { location, .. }
            | InputValueError::UnknownEnumValue { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum ValueKind {
    String,
    Integer,
    Enum,
    Float,
    Object,
    Boolean,
    List,
    Null,
}

impl From<ConstValue> for ValueKind {
    fn from(value: ConstValue) -> Self {
        (&value).into()
    }
}

impl From<&ConstValue> for ValueKind {
    fn from(value: &ConstValue) -> Self {
        match value {
            ConstValue::Null => ValueKind::Null,
            ConstValue::Number(number) if number.is_f64() => ValueKind::Float,
            ConstValue::Number(_) => ValueKind::Integer,
            ConstValue::String(_) => ValueKind::String,
            ConstValue::Boolean(_) => ValueKind::Boolean,
            ConstValue::Binary(_) => ValueKind::String,
            ConstValue::Enum(_) => ValueKind::Enum,
            ConstValue::List(_) => ValueKind::List,
            ConstValue::Object(_) => ValueKind::Object,
        }
    }
}

impl From<Value> for ValueKind {
    fn from(value: Value) -> Self {
        (&value).into()
    }
}

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => ValueKind::Null,
            Value::Number(number) if number.is_f64() => ValueKind::Float,
            Value::Number(_) => ValueKind::Integer,
            Value::String(_) => ValueKind::String,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Binary(_) => ValueKind::String,
            Value::Enum(_) => ValueKind::Enum,
            Value::List(_) => ValueKind::List,
            Value::Object(_) => ValueKind::Object,
            Value::Variable(_) => unreachable!(),
        }
    }
}
