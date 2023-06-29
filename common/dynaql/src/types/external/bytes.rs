use bytes::Bytes;

use crate::{InputValueError, InputValueResult, LegacyScalarType, Scalar, Value};

/// The `Binary` scalar type represents binary data.
#[Scalar(internal)]
impl LegacyScalarType for Bytes {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Binary(data) => Ok(data),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Binary(_))
    }

    fn to_value(&self) -> Value {
        Value::Binary(self.clone())
    }
}
