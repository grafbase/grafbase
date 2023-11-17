use engine_value::ConstValue;
use time::{format_description::well_known::Iso8601, Time};

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct TimeScalar;

impl TimeScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<Time, Error> {
        match value {
            serde_json::Value::String(string) => {
                Time::parse(&string, &Iso8601::TIME).map_err(|e| Error::new(format!("could not parse time: {e}")))
            }
            _ => Err(Error::new("times should be provided as string")),
        }
    }
}

impl<'a> SDLDefinitionScalar<'a> for TimeScalar {
    fn name() -> Option<&'a str> {
        Some("Time")
    }

    fn description() -> Option<&'a str> {
        Some("A time representation")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://en.wikipedia.org/wiki/ISO_8601")
    }
}

impl DynamicParse for TimeScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(time) => Time::parse(time, &Iso8601::TIME).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(time) => Ok(ConstValue::String(time)),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value into an Time",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(time) => {
                Time::parse(&time, &Iso8601::TIME)
                    .map_err(|e| InputValueError::ty_custom("Time", format!("could not parse time: {e}")))?;

                Ok(serde_json::Value::String(time))
            }
            _ => Err(InputValueError::ty_custom("Time", "Cannot parse into a Time")),
        }
    }
}
