use std::str::FromStr;

use engine_value::ConstValue;
use serde_json::Value;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{InputValueError, InputValueResult};

pub struct UnsignedBigIntScalar;

impl<'a> SDLDefinitionScalar<'a> for UnsignedBigIntScalar {
    fn name() -> Option<&'a str> {
        Some("UnsignedBigInt")
    }

    fn description() -> Option<&'a str> {
        Some("An unsigned 64-bit integer value. The value is returned as string.")
    }
}

impl DynamicParse for UnsignedBigIntScalar {
    fn parse(value: ConstValue) -> InputValueResult<Value> {
        match value {
            ConstValue::String(bigint_string) => match u64::from_str(&bigint_string) {
                Ok(_) => Ok(Value::String(bigint_string)),
                Err(_) => Err(InputValueError::ty_custom(
                    "UnsignedBigInt",
                    "Invalid UnsignedBigInt value",
                )),
            },
            _ => Err(InputValueError::ty_custom(
                "UnsignedBigInt",
                "Cannot parse into a UnsignedBigInt",
            )),
        }
    }

    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(ref bigint) => u64::from_str(bigint).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: Value) -> Result<ConstValue, crate::Error> {
        match value {
            Value::String(number) if u64::from_str(&number).is_ok() => Ok(ConstValue::String(number)),
            _ => Err(crate::Error::new(
                "Data violation: Cannot coerce the initial value into a UnsignedBigInt",
            )),
        }
    }
}
