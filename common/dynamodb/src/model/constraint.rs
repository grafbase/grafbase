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
    use super::super::id::ID_SEPARATOR;
    use std::borrow::{Borrow, Cow};
    use std::fmt::Display;

    const CONSTRAINT_PREFIX: &str = "__C";

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ConstraintID<'a> {
        field: Cow<'a, str>,
        ty: Cow<'a, str>,
        value: Cow<'a, serde_json::Value>,
    }

    impl<'a> ConstraintID<'a> {
        pub fn from_owned(ty: String, field: String, value: serde_json::Value) -> Self {
            Self {
                ty: Cow::Owned(ty),
                field: Cow::Owned(field),
                value: Cow::Owned(value),
            }
        }

        pub fn value(&self) -> &serde_json::Value {
            self.value.borrow()
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
            let (prefix, rest) = match origin.split_once(&format!("{CONSTRAINT_PREFIX}{ID_SEPARATOR}")) {
                Some((prefix, rest)) => (prefix, rest),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            if !prefix.is_empty() {
                return Err(ConstraintIDError::NotAConstraint { origin });
            }

            let (ty, rest) = match rest.split_once(ID_SEPARATOR) {
                Some((field, rest)) => (field, rest),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            let (field, value) = match rest.split_once(ID_SEPARATOR) {
                Some((ty, value)) => (ty, value),
                None => return Err(ConstraintIDError::NotAConstraint { origin }),
            };

            Ok(Self {
                ty: Cow::Owned(ty.to_string()),
                field: Cow::Owned(field.to_string()),
                value: Cow::Owned(serde_json::from_str(value)?),
            })
        }
    }

    impl<'a> TryFrom<&'a str> for ConstraintID<'a> {
        type Error = ConstraintIDError;
        fn try_from(origin: &'a str) -> Result<Self, Self::Error> {
            let (prefix, rest) = match origin.split_once(&format!("{CONSTRAINT_PREFIX}{ID_SEPARATOR}")) {
                Some((prefix, rest)) => (prefix, rest),
                None => {
                    return Err(ConstraintIDError::NotAConstraint {
                        origin: origin.to_string(),
                    })
                }
            };

            if !prefix.is_empty() {
                return Err(ConstraintIDError::NotAConstraint {
                    origin: origin.to_string(),
                });
            }

            let (ty, rest) = match rest.split_once(ID_SEPARATOR) {
                Some((field, rest)) => (field, rest),
                None => {
                    return Err(ConstraintIDError::NotAConstraint {
                        origin: origin.to_string(),
                    })
                }
            };

            let (field, value) = match rest.split_once(ID_SEPARATOR) {
                Some((ty, value)) => (ty, value),
                None => {
                    return Err(ConstraintIDError::NotAConstraint {
                        origin: origin.to_string(),
                    })
                }
            };

            Ok(Self {
                ty: Cow::Owned(ty.to_string()),
                field: Cow::Owned(field.to_string()),
                value: Cow::Owned(serde_json::from_str(value)?),
            })
        }
    }

    impl<'a> Display for ConstraintID<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{CONSTRAINT_PREFIX}{ID_SEPARATOR}{}{ID_SEPARATOR}{}{ID_SEPARATOR}{}",
                self.ty.to_lowercase(),
                self.field,
                self.value
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::constraint::db::ConstraintID;

    #[test]
    fn ensure_constraint_new() {
        const TEST_TY: &str = "__C_author_name_\"Val\"";

        let id = ConstraintID::from_owned("Author".into(), "name".into(), serde_json::Value::String("Val".into()));

        assert_eq!(id.to_string(), TEST_TY, "Should give the same result");
    }

    #[test]
    fn ensure_constraint_from_string() {
        const TEST_TY: &str = "__C_author_name_\"Val\"";

        let id = ConstraintID::try_from(TEST_TY.to_string());

        assert!(id.is_ok(), "Id should be transformed");
        assert_eq!(id.unwrap().to_string(), TEST_TY, "Should give the same result");
    }
}
