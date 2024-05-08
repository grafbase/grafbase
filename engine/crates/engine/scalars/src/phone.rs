use engine_value::ConstValue;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct PhoneNumberScalar;

impl<'a> SDLDefinitionScalar<'a> for PhoneNumberScalar {
    fn name() -> Option<&'a str> {
        Some("PhoneNumber")
    }

    fn description() -> Option<&'a str> {
        Some("A phone number. This value is stored as a string. Phone numbers must follow the E.164 format, a general format for international telephone numbers.")
    }
}

impl DynamicParse for PhoneNumberScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(phone) => parse(phone).unwrap_or_default(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(phone) => Ok(ConstValue::String(phone)),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value into an Phone",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(phone) => {
                if parse(&phone).unwrap_or_default() {
                    Ok(serde_json::Value::String(phone))
                } else {
                    Err(InputValueError::ty_custom("PhoneNumber", "Invalid phone number"))
                }
            }
            _ => Err(InputValueError::ty_custom(
                "PhoneNumber",
                "Cannot parse into a PhoneNumber",
            )),
        }
    }
}

/// parse a given string according to the E164 format
fn parse(target: &str) -> Result<bool, Box<dyn std::error::Error + '_>> {
    use nom::{
        bytes::complete::take_while_m_n,
        character::{
            complete::{char, satisfy},
            is_digit,
        },
        combinator::all_consuming,
        error::ErrorKind,
        sequence::tuple,
    };

    pub fn is_char_digit(chr: char) -> bool {
        chr.is_ascii() && is_digit(chr as u8)
    }

    // ^\+[1-9]\d{1,14}$
    all_consuming(tuple::<_, _, (_, ErrorKind), _>((
        char('+'),
        satisfy(|c| ('1'..='9').contains(&c)),
        take_while_m_n(1, 14, is_char_digit),
    )))(target)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use crate::{DynamicParse, PhoneNumberScalar, SDLDefinitionScalar};

    #[test]
    fn should_succeed() {
        let value = serde_json::Value::String("+33612121212".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneNumberScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn should_fail_missing_initial_char() {
        let value = serde_json::Value::String("33612121212".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneNumberScalar::parse(const_value);
        assert!(scalar.is_err());
        insta::assert_debug_snapshot!(scalar);
    }

    #[test]
    fn should_fail_max_length() {
        let value = serde_json::Value::String("+3361212121212121".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneNumberScalar::parse(const_value);
        assert!(scalar.is_err());
    }

    #[test]
    fn should_fail_alphanumeric() {
        let value = serde_json::Value::String("+33612121212121a".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneNumberScalar::parse(const_value);
        assert!(scalar.is_err());
        insta::assert_debug_snapshot!(scalar);
    }

    #[test]
    fn should_fail_starts_with_0() {
        let value = serde_json::Value::String("+03612121212121".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneNumberScalar::parse(const_value);
        assert!(scalar.is_err());
        insta::assert_debug_snapshot!(scalar);
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(PhoneNumberScalar::sdl());
    }
}
