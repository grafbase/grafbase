use chrono::{DateTime, Utc};
use const_format::concatcp;
use dynaql_value::ConstValue;

use crate::{InputValueError, InputValueResult};

/// The `PossibleScalar` enum is the list of possible Scalar usable within dynaql
#[derive(Debug, Clone, Copy)]
pub enum PossibleScalar {
    String,
    Int,
    Float,
    Boolean,
    ID,
    DateTime,
    JSON,
}

const SPECIFIED_BY_DIRECTIVE: &str = r#"
directive @specifiedBy(url: String!) on SCALAR
"#;

const DATETIME_DIRECTIVE: &str = r#"
"""
A date-time string at UTC, such as 2007-12-03T10:15:30Z, is compliant with the date-time format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.

This scalar is a description of an exact instant on the timeline such as the instant that a user account was created.
"""
scalar DateTime @specifiedBy(url: "https://datatracker.ietf.org/doc/html/rfc3339")
"#;

const JSON_DIRECTIVE: &str = r#"
"""
Any JSON value.
"""
scalar JSON
"#;

impl PossibleScalar {
    /// Function to **check** if the inputed value is able to be cast into the expected type.
    /// TODO: In the future, we should also do more than just a check at the request parsing, we
    /// should also allow casting to an expected scalar to be used inside the resolving chain.
    pub(crate) fn check_valid(&self, value: &ConstValue) -> InputValueResult<bool> {
        match (self, value) {
            (Self::String | Self::ID, ConstValue::String(_)) => Ok(true),
            (Self::Boolean, ConstValue::Boolean(_)) => Ok(true),
            (Self::Int, ConstValue::Number(num)) => Ok(!num.is_f64()),
            (Self::Float, ConstValue::Number(_)) => Ok(true),
            (Self::DateTime, ConstValue::String(date)) => {
                date.parse::<DateTime<Utc>>()
                    .map_err(|err| InputValueError::ty_custom("DateTime", err))?;
                Ok(true)
            }
            // TODO: Should ensure that a JSON got string key value
            (Self::JSON, ConstValue::Object(_)) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Generate directives associated
    pub const fn directives() -> &'static str {
        concatcp!(
            SPECIFIED_BY_DIRECTIVE,
            '\n',
            DATETIME_DIRECTIVE,
            '\n',
            JSON_DIRECTIVE
        )
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PossibleScalarErrors {
    #[error("\"{expected_ty}\" is not a proper scalar")]
    NotAScalar { expected_ty: String },
}

impl TryFrom<&str> for PossibleScalar {
    type Error = PossibleScalarErrors;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "String" => Ok(PossibleScalar::String),
            "Int" => Ok(PossibleScalar::Int),
            "Float" => Ok(PossibleScalar::Float),
            "Boolean" => Ok(PossibleScalar::Boolean),
            "ID" => Ok(PossibleScalar::ID),
            "DateTime" => Ok(PossibleScalar::DateTime),
            _ => Err(PossibleScalarErrors::NotAScalar {
                expected_ty: value.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use dynaql_value::ConstValue;

    use super::PossibleScalar;

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

        let scalar = PossibleScalar::JSON.check_valid(&const_value);
        assert!(scalar.is_ok());
    }
}
