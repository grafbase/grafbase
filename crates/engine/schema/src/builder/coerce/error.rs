use federated_graph::Value;

use super::input_value_set::InputValueSetError;

#[derive(Debug, thiserror::Error)]
pub enum ExtensionInputValueError {
    #[error(transparent)]
    InputValue(#[from] InputValueError),
    #[error(transparent)]
    InputValueSetSerror(#[from] InputValueSetError),
    #[error("Unknown type '{name}'")]
    UnknownType { name: String },
    #[error("Type '{name}' is used for an input value but is not a scalar, input object or enum.")]
    NotAnInputType { name: String },
    #[error("Invalid template: {0}")]
    InvalidTemplate(#[from] ramhorns::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum InputValueError {
    #[error("Found a null where we expected a {expected}{path}")]
    UnexpectedNull { expected: String, path: String },
    #[error("Found a {actual} value where we expected a {expected}{path}")]
    MissingList {
        actual: ValueKind,
        expected: String,
        path: String,
    },
    #[error("Found a {actual} value where we expected a '{name}' input object{path}")]
    MissingObject {
        name: String,
        actual: ValueKind,
        path: String,
    },
    #[error("Found a {actual} value where we expected a {expected} scalar{path}")]
    IncorrectScalarType {
        actual: ValueKind,
        expected: String,
        path: String,
    },
    #[error("Found value {actual} which cannot be coerced into a {expected} scalar{path}")]
    IncorrectScalarValue {
        actual: String,
        expected: String,
        path: String,
    },
    #[error("Found a {actual} value where we expected a {enum} enum value{path}")]
    IncorrectEnumValueType {
        r#enum: String,
        actual: ValueKind,
        path: String,
    },
    #[error("Found an unknown enum value '{value}' for the enum {enum}{path}")]
    UnknownEnumValue {
        r#enum: String,
        value: String,
        path: String,
    },
    #[error("Input object {input_object} does not have a field named '{name}'{path}")]
    UnknownInputField {
        input_object: String,
        name: String,
        path: String,
    },
    #[error("Missing required argument named '{0}'")]
    MissingRequiredArgument(String),
    #[error("Unknown argumant named '{0}'")]
    UnknownArgument(String),
    #[error("Used an inaccessible enum value{path}")]
    InaccessibleEnumValue { path: String },
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

impl From<Value> for ValueKind {
    fn from(value: Value) -> Self {
        (&value).into()
    }
}

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(_) => ValueKind::String,
            Value::Int(_) => ValueKind::Integer,
            Value::Float(_) => ValueKind::Float,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Null => ValueKind::Null,
            Value::List(_) => ValueKind::List,
            Value::Object(_) => ValueKind::Object,
            Value::UnboundEnumValue(_) | Value::EnumValue(_) => ValueKind::Enum,
        }
    }
}

impl From<cynic_parser::ConstValue<'_>> for ValueKind {
    fn from(value: cynic_parser::ConstValue) -> Self {
        match value {
            cynic_parser::ConstValue::Int(_) => ValueKind::Integer,
            cynic_parser::ConstValue::Float(_) => ValueKind::Float,
            cynic_parser::ConstValue::String(_) => ValueKind::String,
            cynic_parser::ConstValue::Boolean(_) => ValueKind::Boolean,
            cynic_parser::ConstValue::Null(_) => ValueKind::Null,
            cynic_parser::ConstValue::Enum(_) => ValueKind::Enum,
            cynic_parser::ConstValue::List(_) => ValueKind::List,
            cynic_parser::ConstValue::Object(_) => ValueKind::Object,
        }
    }
}
