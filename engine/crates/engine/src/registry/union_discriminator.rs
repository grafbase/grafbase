use indexmap::set::Union;
use serde_json::Value;

/// Checks if the provided data matches this discriminator
pub fn discriminator_matches(discriminator: &registry_v2::UnionDiscriminator, data: &Value) -> bool {
    use registry_v2::{ScalarKind, UnionDiscriminator};

    match (discriminator, data) {
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

    match discriminator {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::discriminator_matches;
    use registry_v2::UnionDiscriminator;

    #[test]
    fn test_field_present() {
        let discriminator = UnionDiscriminator::FieldPresent("myField".into());

        assert!(discriminator_matches(&discriminator, &json!({ "myField": "whatevs"})));
        assert!(!discriminator_matches(
            &discriminator,
            &json!({ "otherField": "whatevs"})
        ));
    }

    #[test]
    fn test_field_has_value() {
        let discriminator = UnionDiscriminator::FieldHasValue("myField".into(), vec![json!("one"), json!(true)]);

        assert!(discriminator_matches(&discriminator, &json!({ "myField": "one"})));
        assert!(discriminator_matches(&discriminator, &json!({ "myField": true })));
        assert!(!discriminator_matches(&discriminator, &json!({ "myField": false })));
        assert!(!discriminator_matches(&discriminator, &json!({ "myField": "two" })));
        assert!(!discriminator_matches(&discriminator, &json!({ "myField": "null" })));
        assert!(!discriminator_matches(&discriminator, &json!({ "otherField": "one"})));
    }

    #[test]
    fn test_fallback() {
        let discriminator = UnionDiscriminator::Fallback;

        assert!(discriminator_matches(&discriminator, &json!({ "myField": "one"})));
        assert!(discriminator_matches(&discriminator, &json!({ "myField": true })));
        assert!(discriminator_matches(&discriminator, &json!({ "myField": false })));
        assert!(discriminator_matches(&discriminator, &json!({ "myField": "two" })));
        assert!(discriminator_matches(&discriminator, &json!({ "myField": "null" })));
        assert!(discriminator_matches(&discriminator, &json!({ "otherField": "one"})));
    }
}
