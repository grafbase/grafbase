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

    use super::normalize_constraint_value;
    use std::borrow::{Borrow, Cow};
    use std::fmt::Display;

    #[derive(Debug, PartialEq, Eq, Clone, Hash)]
    pub struct ConstraintID<'a> {
        ty: Cow<'a, str>,
        fields: Vec<Cow<'a, str>>,
        pub(super) value: Cow<'a, str>,
    }

    impl<'a> ConstraintID<'a> {
        pub fn from_owned(ty: String, field: String, value: serde_json::Value) -> Self {
            let value =
                base64::prelude::BASE64_STANDARD_NO_PAD.encode(Sha256::digest(normalize_constraint_value(&value)));

            Self {
                ty: Cow::Owned(ty.to_lowercase()),
                fields: vec![Cow::Owned(field)],
                value: Cow::Owned(value),
            }
        }

        #[must_use]
        pub fn with_new_value(self, value: serde_json::Value) -> Self {
            let value =
                base64::prelude::BASE64_STANDARD_NO_PAD.encode(Sha256::digest(normalize_constraint_value(&value)));

            Self {
                value: Cow::Owned(value),
                ..self
            }
        }

        pub fn field(&self) -> &str {
            self.fields.get(0).unwrap().borrow()
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
            if !origin.starts_with("__C#V2#") {
                return parse_constraint_id_v1(&origin).ok_or(ConstraintIDError::NotAConstraint { origin });
            }

            parse_constraint_id_v2(origin.split('#')).ok_or(ConstraintIDError::NotAConstraint { origin })
        }
    }

    impl<'a> TryFrom<&'a str> for ConstraintID<'a> {
        type Error = ConstraintIDError;
        fn try_from(origin: &'a str) -> Result<Self, Self::Error> {
            if !origin.starts_with("__C#V2#") {
                return parse_constraint_id_v1(origin).ok_or(ConstraintIDError::NotAConstraint {
                    origin: origin.to_string(),
                });
            }

            parse_constraint_id_v2(origin.split('#')).ok_or(ConstraintIDError::NotAConstraint {
                origin: origin.to_string(),
            })
        }
    }

    /// Parses an older constraint ID
    fn parse_constraint_id_v1<'a>(origin: &str) -> Option<ConstraintID<'a>> {
        let (ty, rest) = origin.split_once('_')?;

        let (field, value) = rest.split_once('_')?;

        Some(ConstraintID {
            ty: Cow::Owned(ty.to_string()),
            fields: vec![Cow::Owned(field.to_string())],
            value: Cow::Owned(value.to_string()),
        })
    }

    fn parse_constraint_id_v2<'a>(sections: impl Iterator<Item = &'a str>) -> Option<ConstraintID<'static>> {
        // First 2 sections are `__C#V2#` so skip them
        let mut sections = sections.skip(2);

        let ty = sections.next()?;
        let mut fields = sections.collect::<Vec<_>>();
        let value = fields.pop()?;

        if fields.is_empty() {
            return None;
        }

        Some(ConstraintID {
            ty: Cow::Owned(ty.to_string()),
            fields: fields.into_iter().map(|f| Cow::Owned(f.to_string())).collect(),
            value: Cow::Owned(value.to_string()),
        })
    }

    impl<'a> Display for ConstraintID<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "__C#V2#{}#", self.ty)?;
            for field in &self.fields {
                write!(f, "{field}#")?;
            }
            write!(f, "{}", self.value)
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

    #[test]
    fn test_can_load_v1_constraints() {
        const V1_CONSTRAINT: &str = "author_name_Val_1";

        let constraint = ConstraintID::try_from(V1_CONSTRAINT).expect("to be able to parse v1 ConstraintIDs");

        assert_eq!(constraint.ty(), "author");
        assert_eq!(constraint.field(), "name");
        assert_eq!(constraint.value, "Val_1");
    }
}
