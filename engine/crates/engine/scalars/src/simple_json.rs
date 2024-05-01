use engine_value::ConstValue;
use serde::Deserialize;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct SimpleJSON;

impl<'a> SDLDefinitionScalar<'a> for SimpleJSON {
    fn name() -> Option<&'a str> {
        Some("SimpleJSON")
    }

    fn description() -> Option<&'a str> {
        // honestly not sure what this is, this is the comment I could find.
        Some("virtual type for non-JSONB operations (only set)")
    }
}

impl DynamicParse for SimpleJSON {
    fn is_valid(_: &ConstValue) -> bool {
        true
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        ConstValue::deserialize(value).map_err(|error| Error::new(error.to_string()))
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        serde_json::Value::deserialize(value).map_err(|error| InputValueError::ty_custom("JSON", error.to_string()))
    }
}
