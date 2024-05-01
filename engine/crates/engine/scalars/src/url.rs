use std::str::FromStr;

use engine_value::ConstValue;
use url::Url;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct URLScalar;

impl<'a> SDLDefinitionScalar<'a> for URLScalar {
    fn name() -> Option<&'a str> {
        Some("URL")
    }

    fn description() -> Option<&'a str> {
        Some("An URL as defined by RFC1738. For example, `https://grafbase.com/foo/` or `mailto:example@grafbase.com`.")
    }

    fn specified_by() -> Option<&'a str> {
        Some("http://url.spec.whatwg.org/")
    }
}

impl DynamicParse for URLScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(url) => Url::from_str(url).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(url) => Url::from_str(&url)
                .map_err(|err| Error::new(err.to_string()))
                .map(|url| ConstValue::String(url.to_string())),
            _ => Err(Error::new("Data violation: Cannot coerce the initial value to a URL")),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(url) => Url::from_str(&url)
                .map(|url| serde_json::Value::String(url.to_string()))
                .map_err(|err| InputValueError::ty_custom("URL", err)),
            _ => Err(InputValueError::ty_custom("URL", "Cannot parse into a URL")),
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use crate::{DynamicParse, SDLDefinitionScalar, URLScalar};

    #[test]
    fn check_url_valid() {
        let value = serde_json::Value::String("https://grafbase.com".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = URLScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn check_url_mailto_valid() {
        let value = serde_json::Value::String("mailto:anthony@grafbase.com".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = URLScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(URLScalar::sdl());
    }
}
