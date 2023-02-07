use base64::Engine;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintType {
    Unique,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintDefinition {
    pub fields: Vec<String>,
    pub r#type: ConstraintType,
}

pub mod db {
    use super::hash_constraint_values;
    use std::borrow::{Borrow, Cow};
    use std::fmt::Display;

    #[derive(Debug, PartialEq, Eq, Clone, Hash)]
    pub struct ConstraintID<'a> {
        ty: Cow<'a, str>,
        fields: Vec<Cow<'a, str>>,
        pub(super) value: Cow<'a, str>,
    }

    impl<'a> ConstraintID<'a> {
        pub fn new(ty: String, fields: Vec<(String, serde_json::Value)>) -> Self {
            let (fields, values) = fields
                .into_iter()
                .map(|(name, value)| (Cow::Owned(name), value))
                .unzip();

            Self {
                ty: Cow::Owned(ty.to_lowercase()),
                fields,
                value: Cow::Owned(hash_constraint_values(values)),
            }
        }

        #[must_use]
        pub fn with_new_values(self, values: Vec<serde_json::Value>) -> Self {
            assert_eq!(
                values.len(),
                self.fields.len(),
                "Tried to update a ConstraintID with incorrect number of values"
            );

            Self {
                value: Cow::Owned(hash_constraint_values(values)),
                ..self
            }
        }

        pub fn fields(&self) -> impl Iterator<Item = &str> {
            self.fields.iter().map(Cow::borrow)
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

pub fn normalize_constraint_value(value: serde_json::Value) -> String {
    if let serde_json::Value::String(s) = value {
        s
    } else {
        value.to_string()
    }
}

fn hash_constraint_values(mut values: Vec<serde_json::Value>) -> String {
    if values.len() == 1 {
        return base64::prelude::BASE64_STANDARD_NO_PAD
            .encode(Sha256::digest(normalize_constraint_value(values.pop().unwrap())));
    }

    let hash = Sha256::digest(serde_json::to_vec(&values).expect("to be able to serialize constraint values"));

    base64::prelude::BASE64_STANDARD_NO_PAD.encode(hash)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::Value;

    use super::db::ConstraintID;

    #[rstest]
    #[case(vec![("name".into(), Value::String("Val_1".into()))])]
    #[case(vec![
        ("name".into(), Value::String("Val_1".into())),
        ("other".into(), Value::String("hello".into()))
    ])]
    fn test_string_roundtrip(#[case] fields: Vec<(String, Value)>) {
        let id = ConstraintID::new("Author".into(), fields);

        assert_eq!(
            ConstraintID::try_from(id.to_string()).unwrap(),
            id,
            "Constraint ID should survive a roundtrip via a String"
        );
    }

    #[rstest]
    #[case(vec![("name".into(), Value::String("Val_1".into()))])]
    #[case(vec![
        ("name".into(), Value::String("Val_1".into())),
        ("other".into(), Value::String("hello".into()))
    ])]
    fn test_str_roundtrip(#[case] fields: Vec<(String, Value)>) {
        let id = ConstraintID::new("Author".into(), fields);

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
        assert_eq!(constraint.fields().collect::<Vec<_>>(), ["name"]);
        assert_eq!(constraint.value, "Val_1");
    }
}
