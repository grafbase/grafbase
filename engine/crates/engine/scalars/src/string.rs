use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct StringScalar;

impl<'a> SDLDefinitionScalar<'a> for StringScalar {
    fn name() -> Option<&'a str> {
        Some("String")
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

impl DynamicParse for StringScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::String(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => Ok(ConstValue::String(v)),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value to a String",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => Ok(serde_json::Value::String(val)),
            _ => Err(InputValueError::ty_custom("String", "Cannot parse into a String")),
        }
    }
}
