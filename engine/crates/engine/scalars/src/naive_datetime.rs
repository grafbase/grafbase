use chrono::NaiveDateTime;
use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct NaiveDateTimeScalar;

impl NaiveDateTimeScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<NaiveDateTime, Error> {
        Ok(serde_json::from_value::<String>(value)?.parse::<NaiveDateTime>()?)
    }
}

impl<'a> SDLDefinitionScalar<'a> for NaiveDateTimeScalar {
    fn name() -> Option<&'a str> {
        Some("NaiveDateTime")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://datatracker.ietf.org/doc/html/rfc3339")
    }

    fn description() -> Option<&'a str> {
        Some("A date-time string without timezone")
    }
}

impl DynamicParse for NaiveDateTimeScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(val) => val.parse::<NaiveDateTime>().is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => {
                v.parse::<NaiveDateTime>()
                    .map_err(|_| Error::new("Data violation: Cannot coerce the initial value to NaiveDateTime"))?;

                Ok(ConstValue::String(v))
            }
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value to a NaiveDateTime",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => Ok(serde_json::Value::String(val)),
            _ => Err(InputValueError::ty_custom(
                "NaiveDateTime",
                "Cannot parse into a String",
            )),
        }
    }
}
