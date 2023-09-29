use engine_value::ConstValue;
use serde::Deserialize;

use super::super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct FederationAnyScalar;

impl<'a> SDLDefinitionScalar<'a> for FederationAnyScalar {
    fn name() -> Option<&'a str> {
        Some("_Any")
    }

    fn description() -> Option<&'a str> {
        Some("A federation _Any scalar as described in the federation subgraph spec")
    }
}

impl DynamicParse for FederationAnyScalar {
    fn is_valid(value: &ConstValue) -> bool {
        matches!(value, ConstValue::Object(_))
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        if !value.is_object() {
            return Err(Error::new("Expected an _Any to be an object"));
        }
        ConstValue::deserialize(value).map_err(|error| Error::new(error.to_string()))
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        if !value.is_object() {
            return Err(InputValueError::ty_custom("_Any", "Expected an object"));
        }
        serde_json::Value::deserialize(value).map_err(|error| InputValueError::ty_custom("JSON", error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use super::super::super::SDLDefinitionScalar;
    use crate::registry::scalars::{DynamicParse, FederationAnyScalar};

    #[test]
    fn check_json_valid() {
        let value = serde_json::json!({
            "__typename": "User",
            "id": 1
        });

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = FederationAnyScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(FederationAnyScalar::sdl(), @r###"
        """
        A federation _Any scalar as described in the federation subgraph spec
        """
        scalar _Any 
        "###);
    }
}
