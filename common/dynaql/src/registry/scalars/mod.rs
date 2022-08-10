use dynaql_value::ConstValue;

/// The `PossibleScalar` enum is the list of possible Scalar usable within dynaql
#[derive(Debug, Clone, Copy)]
pub enum PossibleScalar {
    String,
    Int,
    Float,
    Boolean,
    ID,
}

impl PossibleScalar {
    pub(crate) fn check_valid(&self, value: &ConstValue) -> bool {
        match (self, value) {
            (Self::String | Self::ID, ConstValue::String(_)) => true,
            (Self::Boolean, ConstValue::Boolean(_)) => true,
            (Self::Int, ConstValue::Number(num)) => !num.is_f64(),
            (Self::Float, ConstValue::Number(_)) => true,
            _ => false,
        }
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
            _ => Err(PossibleScalarErrors::NotAScalar {
                expected_ty: value.to_string(),
            }),
        }
    }
}
