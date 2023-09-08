use std::str::FromStr;

use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::InputValueError;

pub struct BigIntScalar;

impl<'a> SDLDefinitionScalar<'a> for BigIntScalar {
    fn name() -> Option<&'a str> {
        Some("BigInt")
    }

    fn description() -> Option<&'a str> {
        Some("A 64-bit integer value. The value is returned as string.")
    }
}

impl DynamicParse for BigIntScalar {
    fn parse(value: engine_value::ConstValue) -> crate::InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(bigint_string) => match i64::from_str(&bigint_string) {
                Ok(num) => Ok(serde_json::Value::Number(serde_json::Number::from(num))),
                Err(_) => Err(InputValueError::ty_custom("BigInt", "Invalid BigInt value")),
            },
            _ => Err(InputValueError::ty_custom("BigInt", "Cannot parse into a BigInt")),
        }
    }

    fn is_valid(value: &engine_value::ConstValue) -> bool {
        match value {
            ConstValue::String(ref bigint) => i64::from_str(bigint).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<engine_value::ConstValue, crate::Error> {
        match value {
            serde_json::Value::Number(number) => Ok(ConstValue::String(number.to_string())),
            _ => Err(crate::Error::new(
                "Data violation: Cannot coerce the initial value into a BigInt",
            )),
        }
    }
}
