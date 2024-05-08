use engine_value::ConstValue;
use fast_chemail::parse_email;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct EmailScalar;

impl<'a> SDLDefinitionScalar<'a> for EmailScalar {
    fn name() -> Option<&'a str> {
        Some("Email")
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address")
    }

    fn description() -> Option<&'a str> {
        Some("A scalar to validate the email as it is defined in the HTML specification.")
    }
}

impl DynamicParse for EmailScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(val) => parse_email(val).is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(v) => Ok(ConstValue::String(v)),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value to an Email",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(val) => parse_email(&val)
                .map_err(|err| InputValueError::ty_custom("Email", err))
                .map(|()| serde_json::Value::String(val)),
            _ => Err(InputValueError::ty_custom("Email", "Cannot parse into an Email")),
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use crate::{DynamicParse, EmailScalar, SDLDefinitionScalar};

    #[test]
    fn check_mail_valid() {
        let value = serde_json::Value::String("anthony@grafbase.com".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = EmailScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn check_not_mail_valid() {
        let value = serde_json::Value::String("anthony @grafbase.com".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = EmailScalar::parse(const_value);
        assert!(scalar.is_err());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(EmailScalar::sdl());
    }
}
