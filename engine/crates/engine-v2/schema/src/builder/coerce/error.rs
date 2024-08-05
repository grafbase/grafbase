use federated_graph::Value;

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
    #[error("Found a {actual} value where we expected a {r#enum} enum value{path}")]
    IncorrectEnumValueType {
        r#enum: String,
        actual: ValueKind,
        path: String,
    },
    #[error("Unknown enum value '{value}' for enum {r#enum}{path}")]
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
            Value::EnumValue(_) => ValueKind::Enum,
        }
    }
}
