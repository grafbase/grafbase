use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};
use dynaql_value::ConstValue;

pub struct JSONScalar;

impl<'a> SDLDefinitionScalar<'a> for JSONScalar {
    fn name() -> Option<&'a str> {
        Some("JSON")
    }

    fn description() -> Option<&'a str> {
        Some(
            r#"
            A JSON Value
            "#,
        )
    }
}

impl DynamicParse for JSONScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::Object(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            val @ serde_json::Value::Object(_) => {
                ConstValue::from_json(val).map_err(Error::new_with_source)
            }
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value to a JSON",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            val @ ConstValue::Object(_) => {
                ConstValue::into_json(val).map_err(|err| InputValueError::ty_custom("JSON", err))
            }
            _ => Err(InputValueError::ty_custom(
                "JSON",
                "Cannot parse into a JSON",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::registry::scalars::{DynamicParse, JSONScalar};
    use dynaql_value::ConstValue;

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
}
