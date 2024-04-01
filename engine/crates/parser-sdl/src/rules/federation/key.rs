use std::collections::BTreeSet;

use super::field_set::FieldSet;
use crate::rules::{directive::Directive, join_directive::FieldSelection};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyDirective {
    pub fields: FieldSet,
    #[serde(default = "default_to_true")]
    pub resolvable: bool,
    pub select: Option<FieldSelection>,
}

impl KeyDirective {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = vec![];
        if !self.resolvable && self.select.is_some() {
            errors.push("A key with a selection must be resolvable".into());
        }

        if let Some(required_fields) = self.select.as_ref().and_then(|select| select.required_fieldset(&[])) {
            for missing_field in
                fields_from_fieldset(&required_fields).difference(&fields_from_fieldset(&self.fields.0))
            {
                errors.push(format!(
                    "The select for this key requires the field {missing_field} which is not present in the key"
                ));
            }
        }

        errors
    }
}

fn fields_from_fieldset(fieldset: &engine::registry::FieldSet) -> BTreeSet<&str> {
    fieldset.0.iter().map(|field| field.field.as_str()).collect()
}

fn default_to_true() -> bool {
    true
}

impl Directive for KeyDirective {
    fn definition() -> String {
        // Note: technically this is meant to be declared "repeatable"
        // but our parser doesn't seem to support it.
        r"
        directive @key(fields: FieldSet!, resolvable: Boolean = true, select: FieldSelection) on OBJECT | INTERFACE
        "
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::assert_validation_error;

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

        insta::assert_snapshot!(registry.export_sdl(true));
    }

    #[test]
    fn entity_with_selection() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: ID!): User! @resolver(name: "blah")
            }

            type User @key(fields: "id", select: "blah(id: $id)" ) {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_json_snapshot!(registry.federation_entities.get("User").unwrap().keys, @r###"
        [
          {
            "selections": [
              {
                "field": "id",
                "selections": []
              }
            ],
            "resolver": {
              "Join": {
                "fields": [
                  [
                    "blah",
                    [
                      [
                        "id",
                        {
                          "$var": "id"
                        }
                      ]
                    ]
                  ]
                ]
              }
            }
          }
        ]
        "###);
    }

    #[test]
    fn entity_with_selection_but_optional_joined_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: ID!): User @resolver(name: "blah")
            }

            type User @key(fields: "id", select: "blah(id: $id)" ) {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_json_snapshot!(registry.federation_entities.get("User").unwrap().keys, @r###"
        [
          {
            "selections": [
              {
                "field": "id",
                "selections": []
              }
            ],
            "resolver": {
              "Join": {
                "fields": [
                  [
                    "blah",
                    [
                      [
                        "id",
                        {
                          "$var": "id"
                        }
                      ]
                    ]
                  ]
                ]
              }
            }
          }
        ]
        "###);
    }

    #[test]
    fn user_can_only_use_fields_in_key_in_selection() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: ID!): User @resolver(name: "blah")
            }

            type User @key(fields: "id", select: "blah(id: $other)" ) {
                id: ID!
                other: String!
            }
            "#,
            "The select for this key requires the field other which is not present in the key"
        );
    }

    #[test]
    fn unresolvable_federation_keys_with_select() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false, select: "blah" ) {
                id: ID!
            }
            "#,
            "A key with a selection must be resolvable"
        );
    }

    #[test]
    fn test_key_resolver_type_mismatches() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: ID!): [User!]! @resolver(name: "blah")
            }

            type User @key(fields: "id", select: "blah(id: $id)" ) {
                id: ID!
            }
            "#,
            "federation key `id` on the type User is trying to join with Query.blah, but those fields do not have compatible types: '[User!]!' and 'User'"
        );
    }
}
