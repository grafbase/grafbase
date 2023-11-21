use std::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};

use crate::{InputValueError, InputValueResult, LegacyScalarType, Number, Scalar, Value};

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroI8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_i64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i64::from(i8::MIN) || n > i64::from(i8::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i8::MIN,
                        i8::MAX
                    )));
                }
                Ok(NonZeroI8::new(n as i8).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(i64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroI16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_i64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i64::from(i16::MIN) || n > i64::from(i16::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i16::MIN,
                        i16::MAX
                    )));
                }
                Ok(NonZeroI16::new(n as i16).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(i64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroI32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_i64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i64::from(i32::MIN) || n > i64::from(i32::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i32::MIN,
                        i32::MAX
                    )));
                }
                Ok(NonZeroI32::new(n as i32).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(i64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroI64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_i64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                Ok(NonZeroI64::new(n).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get()))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroU8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_u64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u64::from(u8::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u8::MAX
                    )));
                }
                Ok(NonZeroU8::new(n as u8).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(u64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroU16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_u64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u64::from(u16::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u16::MAX
                    )));
                }
                Ok(NonZeroU16::new(n as u16).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(u64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroU32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_u64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u64::from(u32::MAX) || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u32::MAX
                    )));
                }
                Ok(NonZeroU32::new(n as u32).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(u64::from(self.get())))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl LegacyScalarType for NonZeroU64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n.as_u64().ok_or_else(|| InputValueError::from("Invalid number"))?;
                Ok(NonZeroU64::new(n).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get()))
    }
}
