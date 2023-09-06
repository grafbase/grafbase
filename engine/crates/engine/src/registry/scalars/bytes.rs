use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use engine_value::ConstValue;
use serde_json::Value;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::InputValueError;
pub struct BytesScalar;

impl<'a> SDLDefinitionScalar<'a> for BytesScalar {
    fn name() -> Option<&'a str> {
        Some("Bytes")
    }

    fn description() -> Option<&'a str> {
        Some("A base64-encoded set of bytes. The value is returned as string.")
    }
}

impl DynamicParse for BytesScalar {
    fn parse(value: ConstValue) -> crate::InputValueResult<Value> {
        match value {
            ConstValue::String(bytes_string) => match STANDARD_NO_PAD.decode(&bytes_string) {
                Ok(_) => Ok(Value::String(bytes_string)),
                Err(_) => Err(InputValueError::ty_custom("Bytes", "Invalid Bytes value")),
            },
            _ => Err(InputValueError::ty_custom("Bytes", "Cannot parse into Bytes")),
        }
    }

    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(ref bytes_string) => STANDARD_NO_PAD.decode(bytes_string).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: Value) -> Result<ConstValue, crate::Error> {
        match value {
            Value::String(bytes) => match STANDARD_NO_PAD.decode(&bytes) {
                Ok(_) => Ok(ConstValue::String(bytes)),
                Err(_) => {
                    let bytes = STANDARD_NO_PAD.encode(bytes.as_bytes());
                    Ok(ConstValue::String(bytes))
                }
            },
            _ => Err(crate::Error::new(
                "Data violation: Cannot coerce the initial value into a Bytes",
            )),
        }
    }
}
