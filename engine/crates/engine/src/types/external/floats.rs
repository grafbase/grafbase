use crate::{InputValueError, InputValueResult, LegacyScalarType, Number, Scalar, Value};

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl LegacyScalarType for f32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => Ok(n.as_f64().ok_or_else(|| InputValueError::from("Invalid number"))? as Self),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(_))
    }

    fn to_value(&self) -> Value {
        match Number::from_f64(f64::from(*self)) {
            Some(n) => Value::Number(n),
            None => Value::Null,
        }
    }
}

/// The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).
#[Scalar(internal, name = "Float")]
impl LegacyScalarType for f64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => Ok(n.as_f64().ok_or_else(|| InputValueError::from("Invalid number"))? as Self),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(_))
    }

    fn to_value(&self) -> Value {
        match Number::from_f64(*self) {
            Some(n) => Value::Number(n),
            None => Value::Null,
        }
    }
}
