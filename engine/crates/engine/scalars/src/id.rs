use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct IDScalar;

impl<'a> SDLDefinitionScalar<'a> for IDScalar {
    fn name() -> Option<&'a str> {
        Some("ID")
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

impl DynamicParse for IDScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::String(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => Ok(ConstValue::String(v)),
            v => Err(Error::new(format!(
                "Data violation: Cannot coerce the initial value to an ID, got {v}"
            ))),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => Ok(serde_json::Value::String(val)),
            _ => Err(InputValueError::ty_custom("ID", "Cannot parse into an ID")),
        }
    }
}
