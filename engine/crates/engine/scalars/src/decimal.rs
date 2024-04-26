use std::str::FromStr;

use engine_value::ConstValue;
use rust_decimal::Decimal;

use crate::{DynamicParse, InputValueError, SDLDefinitionScalar};

pub struct DecimalScalar;

impl<'a> SDLDefinitionScalar<'a> for DecimalScalar {
    fn name() -> Option<&'a str> {
        Some("Decimal")
    }

    fn description() -> Option<&'a str> {
        Some("A Decimal value. The value is returned as string.")
    }
}

impl DynamicParse for DecimalScalar {
    fn parse(value: crate::ConstValue) -> crate::InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(decimal_string) => {
                if Decimal::from_str(&decimal_string).is_err() {
                    return Err(InputValueError::ty_custom("Decimal", "Invalid Decimal value"));
                }

                Ok(serde_json::Value::String(decimal_string))
            }
            _ => Err(InputValueError::ty_custom("Decimal", "Cannot parse into a Decimal")),
        }
    }

    fn is_valid(value: &crate::ConstValue) -> bool {
        match value {
            ConstValue::String(ref decimal) => Decimal::from_str(decimal).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<crate::ConstValue, crate::Error> {
        match value {
            serde_json::Value::String(number) => Ok(ConstValue::String(number)),
            serde_json::Value::Number(number) => Ok(ConstValue::String(number.to_string())),
            _ => Err(crate::Error::new(
                "Data violation: Cannot coerce the initial value into a Decimal",
            )),
        }
    }
}
