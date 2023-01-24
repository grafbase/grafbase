#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintType {
    Unique,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDefinition {
    pub field: String,
    pub r#type: ConstraintType,
}

pub mod db {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    use super::super::ID_SEPARATOR;
    use super::normalize_constraint_value;
    use std::borrow::{Borrow, Cow};
    use std::fmt::Display;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ConstraintID<'a> {
        ty: Cow<'a, str>,
        field: Cow<'a, str>,
        value: Cow<'a, str>,
    }

    impl<'a> ConstraintID<'a> {
        pub fn from_owned(ty: String, field: String, value: serde_json::Value) -> Self {
            let value = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(Sha256::digest(normalize_constraint_value(&value)));

            Self {
                ty: Cow::Owned(ty),
                field: Cow::Owned(field),
                value: Cow::Owned(value),
            }
        }

        pub fn with_new_value(self, value: serde_json::Value) -> Self {
            let value = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(Sha256::digest(normalize_constraint_value(&value)));

            Self {
                value: Cow::Owned(value),
                ..self
            }
        }

        pub fn field(&self) -> &str {
            self.field.borrow()
        }

        pub fn ty(&self) -> Cow<'a, str> {
            self.ty.clone()
        }
    }

    #[derive(thiserror::Error, Debug)]
    pub enum ConstraintIDError {
        #[error("An internal error happened in the modelisation")]
        NotAConstraint { origin: String },
        #[error("An internal error happened in the modelisation")]
        ValueNotDeserializable(#[from] serde_json::Error),
    }

    impl<'a> TryFrom<String> for ConstraintID<'a> {
        type Error = ConstraintIDError;
        fn try_from(origin: String) -> Result<Self, Self::Error> {
            let (ty, rest) = match origin.split_once(ID_SEPARATOR) {
                Some((ty, rest)) => (ty, rest),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            let (field, value) = match rest.split_once(ID_SEPARATOR) {
                Some((field, value)) => (field, value),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            Ok(Self {
                ty: Cow::Owned(ty.to_string()),
                field: Cow::Owned(field.to_string()),
                value: Cow::Owned(value.to_string()),
            })
        }
    }

    impl<'a> TryFrom<&'a str> for ConstraintID<'a> {
        type Error = ConstraintIDError;
        fn try_from(origin: &'a str) -> Result<Self, Self::Error> {
            let (ty, rest) = match origin.split_once(ID_SEPARATOR) {
                Some((ty, rest)) => (ty, rest),
                None => {
                    return Err(ConstraintIDError::NotAConstraint {
                        origin: origin.to_string(),
                    })
                }
            };

            let (field, value) = match rest.split_once(ID_SEPARATOR) {
                Some((field, value)) => (field, value),
                None => {
                    return Err(ConstraintIDError::NotAConstraint {
                        origin: origin.to_string(),
                    })
                }
            };

            Ok(Self {
                ty: Cow::Owned(ty.to_string()),
                field: Cow::Owned(field.to_string()),
                value: Cow::Owned(value.to_string()),
            })
        }
    }

    impl<'a> Display for ConstraintID<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}{ID_SEPARATOR}{}{ID_SEPARATOR}{}", self.ty, self.field, self.value)
        }
    }
}

pub fn normalize_constraint_value(value: &serde_json::Value) -> String {
    if value.is_string() {
        value.as_str().unwrap().to_owned()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::db::ConstraintID;

    #[test]
    fn test_string_roundtrip() {
        let id = ConstraintID::from_owned(
            "Author".into(),
            "name".into(),
            serde_json::Value::String("Val_1".into()),
        );

        assert_eq!(
            ConstraintID::try_from(id.to_string()).unwrap(),
            id,
            "Constraint ID should survive a roundtrip via a String"
        );
    }

    #[test]
    fn test_str_roundtrip() {
        let id = ConstraintID::from_owned(
            "Author".into(),
            "name".into(),
            serde_json::Value::String("Val_1".into()),
        );

        assert_eq!(
            ConstraintID::try_from(id.to_string().as_str()).unwrap(),
            id,
            "Constraint ID should survive a roundtrip via a str"
        );
    }
}
