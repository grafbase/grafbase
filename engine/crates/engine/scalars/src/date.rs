use std::str::FromStr;

use chrono::{DateTime, NaiveDate};
use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

const DATE_FORMAT: &str = "%Y-%m-%d";

pub struct DateScalar;

impl DateScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<NaiveDate, Error> {
        Ok(NaiveDate::from_str(&serde_json::from_value::<String>(value)?)?)
    }
}

impl<'a> SDLDefinitionScalar<'a> for DateScalar {
    fn name() -> Option<&'a str> {
        Some("Date")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://datatracker.ietf.org/doc/html/rfc3339#appendix-A")
    }

    fn description() -> Option<&'a str> {
        Some(
            r"
A date string, such as 2007-12-03, is compliant with the full-date format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for the representation of dates and times using the Gregorian calendar.

This scalar is a description of the date, as used for birthdays for example. It cannot represent an instant on the timeline.",
        )
    }
}

impl DynamicParse for DateScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(val) => NaiveDate::parse_from_str(val, DATE_FORMAT).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => {
                // mongo sends everything in rfc3339, dynamo stores in DATE_FORMAT

                if NaiveDate::parse_from_str(&v, DATE_FORMAT).is_ok() {
                    return Ok(ConstValue::String(v));
                }

                if let Ok(datetime) = DateTime::parse_from_rfc3339(&v) {
                    return Ok(ConstValue::String(datetime.naive_local().date().to_string()));
                }

                Err(Error::new("Data violation: Cannot coerce the initial value to a Date"))
            }
            _ => Err(Error::new("Data violation: Cannot coerce the initial value to a Date")),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => {
                if NaiveDate::parse_from_str(&val, DATE_FORMAT).is_ok() {
                    Ok(serde_json::Value::String(val))
                } else {
                    Err(InputValueError::ty_custom("Date", "Cannot parse into a Date"))
                }
            }
            _ => Err(InputValueError::ty_custom("Date", "Cannot parse into a Date")),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::{super::SDLDefinitionScalar, DateScalar};

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(DateScalar::sdl());
    }
}
