use engine_value::ConstValue;
use serde_json::Number;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct FloatScalar;

impl<'a> SDLDefinitionScalar<'a> for FloatScalar {
    fn name() -> Option<&'a str> {
        Some("Float")
    }

    fn specified_by() -> Option<&'a str> {
        None
    }

    fn description() -> Option<&'a str> {
        None
    }

    fn internal() -> bool {
        true
    }
}

impl DynamicParse for FloatScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::Number(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::Number(v) => {
                let v = v
                    .as_f64()
                    .and_then(Number::from_f64)
                    .ok_or_else(|| Error::new("Data violation: Cannot coerce the initial value to a Float"))?;

                Ok(ConstValue::Number(v))
            }
            _ => Err(Error::new("Data violation: Cannot coerce the initial value to a Float")),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::Number(val) => Ok(serde_json::Value::Number(val)),
            _ => Err(InputValueError::ty_custom("Float", "Cannot parse into a Float")),
        }
    }
}
