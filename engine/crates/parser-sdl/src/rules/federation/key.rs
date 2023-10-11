use engine::registry::federation::Selection;
use serde::Deserializer;

use crate::rules::directive::Directive;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyDirective {
    #[serde(deserialize_with = "deserialize_selections")]
    pub fields: Vec<Selection>,
    #[serde(default = "default_to_true")]
    pub resolvable: bool,
}

fn default_to_true() -> bool {
    true
}

impl Directive for KeyDirective {
    fn definition() -> String {
        // Note: technically this is meant to be declared "repeatable"
        // but our parser doesn't seem to support it.
        r#"
        directive @key(fields: FieldSet!, resolvable: Boolean = true) on OBJECT | INTERFACE

        "#
        .to_string()
    }
}

fn deserialize_selections<'de, D>(deserializer: D) -> Result<Vec<Selection>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Selection>;

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // This whole implementation is extremely naive and doesn't support a lot of stuff.
            // Will fix in https://linear.app/grafbase/issue/GB-5086
            if value.contains('{') {
                return Err(E::custom("nested fields in keys aren't supported at the moment"));
            }

            Ok(value
                .split(' ')
                .map(|field| Selection {
                    field: field.to_string(),
                    selections: vec![],
                })
                .collect())
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "a string in FieldSet format")
        }
    }

    deserializer.deserialize_str(Visitor)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn unresolvable_federation_entity_on_normal_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true));
    }

    macro_rules! assert_validation_error {
        ($schema:literal, $expected_message:literal) => {
            assert_matches!(
                crate::parse_registry($schema)
                    .err()
                    .and_then(crate::Error::validation_errors)
                    // We don't care whether there are more errors or not.
                    // It only matters that we find the expected error.
                    .and_then(|errors| errors.into_iter().next()),
                Some(crate::RuleError { message, .. }) => {
                    assert_eq!(message, $expected_message);
                }
            )
        };
    }

    #[test]
    fn resolvable_basic_types_not_allowed() {
        assert_validation_error!(
            r#"
                extend schema @federation(version: "2.3")

                type User @key(fields: "id") {
                    id: ID!
                }
            "#,
            "Found a resolvable key on a basic type, which is currently unsupported"
        );
    }
}
