use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct IntScalar;

impl<'a> SDLDefinitionScalar<'a> for IntScalar {
    fn name() -> Option<&'a str> {
        Some("Int")
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

impl DynamicParse for IntScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::Number(v) => !v.is_f64(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::Number(v) => Ok(ConstValue::Number(v)),
            _ => Err(Error::new("Data violation: Cannot coerce the initial value to a Int")),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::Number(v) => Ok(serde_json::Value::Number(v)),
            _ => Err(InputValueError::ty_custom("Int", "Cannot parse into a Int")),
        }
    }
}
