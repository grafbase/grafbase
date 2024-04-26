use chrono::{DateTime, TimeZone, Utc};
use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

// TODO: Input coercion to accept either ms or a date
pub struct TimestampScalar;

impl TimestampScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<DateTime<Utc>, Error> {
        match Utc.timestamp_millis_opt(serde_json::from_value(value)?) {
            chrono::LocalResult::Single(dt) => Ok(dt),
            _ => Err(Error::new("Invalid milliseconds")),
        }
    }
}

impl<'a> SDLDefinitionScalar<'a> for TimestampScalar {
    fn name() -> Option<&'a str> {
        Some("Timestamp")
    }

    fn description() -> Option<&'a str> {
        Some("A Unix Timestamp with milliseconds precision.")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap04.html#tag_04_16")
    }
}

impl DynamicParse for TimestampScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::Number(ms) => ms.is_u64(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::Number(ms) => {
                if ms.is_u64() {
                    Ok(ConstValue::Number(ms))
                } else {
                    Err(Error::new("Cannot coerce the initial value into a valid Timestamp"))
                }
            }
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value into an Timestamp",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::Number(ms) => {
                if ms.is_u64() {
                    Ok(serde_json::Value::Number(ms))
                } else {
                    Err(InputValueError::ty_custom(
                        "Timestamp",
                        "You have to provide an unsigned integer.",
                    ))
                }
            }
            _ => Err(InputValueError::ty_custom("Timestamp", "Cannot parse into a Timestamp")),
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use crate::{DynamicParse, SDLDefinitionScalar, TimestampScalar};

    #[test]
    fn check_test_timestamp() {
        let value = serde_json::json!(1_232_231);

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = TimestampScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(TimestampScalar::sdl());
    }
}
