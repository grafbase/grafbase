use crate::rules::directive::Directive;

use super::field_set::FieldSet;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyDirective {
    pub fields: FieldSet,
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
}
