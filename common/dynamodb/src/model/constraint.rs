#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstraintType {
    Unique,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintDefinition {
    pub field: String,
    pub r#type: ConstraintType,
}

pub mod db {
    use std::borrow::Cow;
    use std::fmt::Display;

    const CONSTRAINT_PREFIX: &str = "__C";

    pub struct ConstraintID<'a> {
        ty: Cow<'a, str>,
        value: Cow<'a, str>,
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ConstraintIDError {
        #[error("An internal error happened in the modelisation")]
        NotAConstraint { origin: String },
    }

    impl<'a> TryFrom<String> for ConstraintID<'a> {
        type Error = ConstraintIDError;
        fn try_from(origin: String) -> Result<Self, Self::Error> {
            let (prefix, rest) = match origin.split_once('#') {
                Some((prefix, rest)) => (prefix, rest),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            if prefix != CONSTRAINT_PREFIX {
                return Err(ConstraintIDError::NotAConstraint { origin });
            }

            let (ty, value) = match rest.split_once('#') {
                Some((ty, value)) => (ty, value),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            Ok(Self {
                ty: Cow::Owned(ty.to_string()),
                value: Cow::Owned(value.to_string()),
            })
        }
    }

    impl<'a> Display for ConstraintID<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{CONSTRAINT_PREFIX}#{}#{}", self.ty, self.value)
        }
    }
}
