use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use engine_value::ConstValue;
use serde_json::Value;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::InputValueError;

pub struct HexBytesScalar;

impl<'a> SDLDefinitionScalar<'a> for HexBytesScalar {
    fn name() -> Option<&'a str> {
        Some("HexBytes")
    }

    fn description() -> Option<&'a str> {
        Some("Bytes stored as hex. The value is returned as base64-encoded string.")
    }
}

impl DynamicParse for HexBytesScalar {
    fn parse(value: ConstValue) -> crate::InputValueResult<Value> {
        match value {
            ConstValue::String(bytes_string) => match STANDARD_NO_PAD.decode(bytes_string) {
                Ok(bytes_vector) => Ok(Value::String(format!("\\x{}", hex::encode(bytes_vector)))),
                Err(_) => Err(InputValueError::ty_custom("Bytes", "Invalid HexBytes value")),
            },
            _ => Err(InputValueError::ty_custom("Bytes", "Cannot parse into HexBytes")),
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
            Value::String(bytes) => hex::decode(&bytes[2..])
                .map(|bytes| STANDARD_NO_PAD.encode(bytes))
                .map(ConstValue::String)
                .map_err(|e| {
                    crate::Error::new(format!(
                        "Data violation: Cannot coerce the initial value into HexBytes: {e}"
                    ))
                }),
            _ => Err(crate::Error::new(
                "Data violation: Cannot coerce the initial value into HexBytes",
            )),
        }
    }
}
