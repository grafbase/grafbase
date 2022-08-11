use chrono::{DateTime, Utc};
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
}

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
            _ => Ok(false),
        }
    }

    /// Generate directives associated
    pub const fn directives() -> &'static str {
        r#"
        directive @specifiedBy(url: String!) on SCALAR

        """
        A date-time string at UTC, such as 2007-12-03T10:15:30Z, is compliant with the date-time format outlined in section 5.6 of the RFC 3339 profile of the ISO 8601 standard for representation of dates and times using the Gregorian calendar.

        This scalar is a description of an exact instant on the timeline such as the instant that a user account was created.
        """
        scalar DateTime @specifiedBy(url: "https://datatracker.ietf.org/doc/html/rfc3339")
        "#
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
