use crate::{InputValueError, InputValueResult, LegacyScalarType, Scalar, Value};

/// The `Boolean` scalar type represents `true` or `false`.
#[Scalar(internal, name = "Boolean")]
impl LegacyScalarType for bool {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Boolean(_))
    }

    fn to_value(&self) -> Value {
        Value::Boolean(*self)
    }
}
