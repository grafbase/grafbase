use serde_json::Value;

/// Specifies how to determine which possible_type a union represents.
///
/// This is mostly useful for remote unions (such as those `@openapi` generates)
/// For non-remote unions we'd generally have this info in the DB.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnionDiscriminator {
    /// If the named field is present then this is the correct variant
    FieldPresent(String),
    /// This is the correct variant if the given field has one of the provided values
    FieldHasValue(String, Vec<Value>),
    /// This is the correct variant if the input is of a particular type
    IsAScalar(ScalarKind),
    /// Fallback on this type if no others match
    Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScalarKind {
    String,
    Number,
    Boolean,
}

impl UnionDiscriminator {
    /// Checks if the provided data matches this discriminator
    pub fn matches(&self, data: &Value) -> bool {
        match (self, data) {
            (UnionDiscriminator::Fallback, _) => return true,
            (UnionDiscriminator::IsAScalar(ScalarKind::Boolean), Value::Bool(_)) => return true,
            (UnionDiscriminator::IsAScalar(ScalarKind::String), Value::String(_)) => return true,
            (UnionDiscriminator::IsAScalar(ScalarKind::Number), Value::Number(_)) => return true,
            (UnionDiscriminator::IsAScalar(_), _) => return false,
            _ => {}
        }

        let Value::Object(object) = data else {
            // The other discriminators only support objects.
            return false;
        };

        match self {
            UnionDiscriminator::FieldPresent(field) => object.contains_key(field),
            UnionDiscriminator::FieldHasValue(field, expected_values) => {
                let Some(actual_value) = object.get(field) else {
                    return false;
                };

                expected_values
                    .iter()
                    .any(|expected_value| expected_value == actual_value)
            }
            UnionDiscriminator::Fallback | UnionDiscriminator::IsAScalar(_) => {
                unreachable!()
            }
        }
    }
}

// serde_json::Value isn't Hash so we have to do this by hand
impl std::hash::Hash for UnionDiscriminator {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            UnionDiscriminator::FieldPresent(field) => field.hash(state),
            UnionDiscriminator::FieldHasValue(field, values) => {
                field.hash(state);
                for value in values {
                    value.to_string().hash(state);
                }
            }
            UnionDiscriminator::IsAScalar(inner) => inner.hash(state),
            UnionDiscriminator::Fallback => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_field_present() {
        let discriminator = UnionDiscriminator::FieldPresent("myField".into());

        assert!(discriminator.matches(&json!({ "myField": "whatevs"})));
        assert!(!discriminator.matches(&json!({ "otherField": "whatevs"})));
    }

    #[test]
    fn test_field_has_value() {
        let discriminator = UnionDiscriminator::FieldHasValue("myField".into(), vec![json!("one"), json!(true)]);

        assert!(discriminator.matches(&json!({ "myField": "one"})));
        assert!(discriminator.matches(&json!({ "myField": true })));
        assert!(!discriminator.matches(&json!({ "myField": false })));
        assert!(!discriminator.matches(&json!({ "myField": "two" })));
        assert!(!discriminator.matches(&json!({ "myField": "null" })));
        assert!(!discriminator.matches(&json!({ "otherField": "one"})));
    }

    #[test]
    fn test_fallback() {
        let discriminator = UnionDiscriminator::Fallback;

        assert!(discriminator.matches(&json!({ "myField": "one"})));
        assert!(discriminator.matches(&json!({ "myField": true })));
        assert!(discriminator.matches(&json!({ "myField": false })));
        assert!(discriminator.matches(&json!({ "myField": "two" })));
        assert!(discriminator.matches(&json!({ "myField": "null" })));
        assert!(discriminator.matches(&json!({ "otherField": "one"})));
    }
}
