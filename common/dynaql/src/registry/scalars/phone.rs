use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};
use dynaql_value::ConstValue;
use phonenumber as _;

// TODO: Input coercion to accept either ms or a date
pub struct PhoneScalar;

impl<'a> SDLDefinitionScalar<'a> for PhoneScalar {
    fn name() -> Option<&'a str> {
        Some("Phone")
    }

    fn description() -> Option<&'a str> {
        Some("A phone number. This value is stored as a string. Phone numbers can contain either spaces or hyphens to separate digit groups. Phone numbers without a country code are assumed to be US/North American numbers adhering to the North American Numbering Plan (NANP).")
    }
}

impl DynamicParse for PhoneScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(phone) => phonenumber::parse(None, phone)
                .map(|phone| phone.is_valid())
                .unwrap_or_default(),
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
                let e164_phone = phonenumber::parse(None, phone)
                    .map_err(|err| InputValueError::ty_custom("Phone", err))?;

                if e164_phone.is_valid() {
                    Ok(serde_json::Value::String(
                        e164_phone
                            .format()
                            .mode(phonenumber::Mode::International)
                            .to_string(),
                    ))
                } else {
                    Err(InputValueError::ty_custom("Phone", "Invalid phone number"))
                }
            }
            _ => Err(InputValueError::ty_custom(
                "Phone",
                "Cannot parse into a Phone",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::SDLDefinitionScalar;
    use crate::registry::scalars::{DynamicParse, PhoneScalar};
    use dynaql_value::ConstValue;
    use insta::assert_snapshot;

    #[test]
    fn check_test_phonenumber() {
        let value = serde_json::Value::String("+33612121212".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn check_test_fail_phonenumber() {
        let value = serde_json::Value::String("+3361212121".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = PhoneScalar::parse(const_value);
        assert!(scalar.is_err());
        insta::assert_debug_snapshot!(scalar);
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(PhoneScalar::sdl());
    }
}
