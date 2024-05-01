use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct BooleanScalar;

impl<'a> SDLDefinitionScalar<'a> for BooleanScalar {
    fn name() -> Option<&'a str> {
        Some("Boolean")
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

impl DynamicParse for BooleanScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::Boolean(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::Bool(v) => Ok(ConstValue::Boolean(v)),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value to a Boolean",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::Boolean(v) => Ok(serde_json::Value::Bool(v)),
            _ => Err(InputValueError::ty_custom("Boolean", "Cannot parse into a Boolean")),
        }
    }
}
