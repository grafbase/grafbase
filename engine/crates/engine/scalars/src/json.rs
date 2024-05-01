use engine_value::ConstValue;
use serde::Deserialize;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct JSONScalar;

impl<'a> SDLDefinitionScalar<'a> for JSONScalar {
    fn name() -> Option<&'a str> {
        Some("JSON")
    }

    fn description() -> Option<&'a str> {
        Some("A JSON Value")
    }
}

impl DynamicParse for JSONScalar {
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

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use crate::{DynamicParse, JSONScalar, SDLDefinitionScalar};

    #[test]
    fn check_json_valid() {
        let value = serde_json::json!({
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ]
            }
        });

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = JSONScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(JSONScalar::sdl());
    }
}
