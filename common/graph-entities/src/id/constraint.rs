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
    use super::super::ID_SEPARATOR;
    use super::normalize_constraint_value;
    use std::borrow::{Borrow, Cow};
    use std::fmt::Display;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ConstraintID<'a> {
        field: Cow<'a, str>,
        ty: Cow<'a, str>,
        value: Cow<'a, str>,
    }

    impl<'a> ConstraintID<'a> {
        pub fn from_owned(ty: String, field: String, value: serde_json::Value) -> Self {
            let value = normalize_constraint_value(&value);

            Self {
                ty: Cow::Owned(ty),
                field: Cow::Owned(field),
                value: Cow::Owned(value),
            }
        }

        pub fn value(&self) -> &str {
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
            write!(
                f,
                "{}{ID_SEPARATOR}{}{ID_SEPARATOR}{}",
                self.ty.to_lowercase(),
                self.field,
                self.value
            )
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
    fn ensure_constraint_new() {
        const TEST_TY: &str = "author_name_Val_1";

        let id = ConstraintID::from_owned(
            "Author".into(),
            "name".into(),
            serde_json::Value::String("Val_1".into()),
        );

        assert_eq!(id.to_string(), TEST_TY, "Should give the same result");
    }

    #[test]
    fn ensure_constraint_from_string() {
        const TEST_TY: &str = "author_name_Val";

        let id = ConstraintID::try_from(TEST_TY.to_string());

        assert!(id.is_ok(), "Id should be transformed");
        assert_eq!(id.unwrap().to_string(), TEST_TY, "Should give the same result");
    }
}
