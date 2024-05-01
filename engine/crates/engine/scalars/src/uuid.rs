use engine_value::ConstValue;
use uuid::Uuid;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct UuidScalar;

impl UuidScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<Uuid, Error> {
        Ok(serde_json::from_value::<String>(value)?.parse::<Uuid>()?)
    }
}

impl<'a> SDLDefinitionScalar<'a> for UuidScalar {
    fn name() -> Option<&'a str> {
        Some("Uuid")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://en.wikipedia.org/wiki/Universally_unique_identifier")
    }

    fn description() -> Option<&'a str> {
        Some("Universally unique identifier")
    }
}

impl DynamicParse for UuidScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(val) => val.parse::<Uuid>().is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => {
                v.parse::<Uuid>()
                    .map_err(|_| Error::new("Data violation: Cannot coerce the initial value to a Uuid"))?;

                Ok(ConstValue::String(v))
            }
            _ => Err(Error::new("Data violation: Cannot coerce the initial value to a Uuid")),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => {
                val.parse::<Uuid>()
                    .map_err(|err| InputValueError::ty_custom("Uuid", err))?;

                Ok(serde_json::Value::String(val))
            }
            _ => Err(InputValueError::ty_custom("Uuid", "Cannot parse into a String")),
        }
    }
}
