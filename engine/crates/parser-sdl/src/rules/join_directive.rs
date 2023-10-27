use std::collections::{BTreeSet, HashMap};

use engine::registry::{field_set::Selection, resolvers::join::JoinResolver, FieldSet};
use engine_parser::{parse_field, types::ConstDirective, Positioned};
use engine_value::{Name, Value};
use serde::de::Error;

use crate::directive_de::parse_directive;

use super::{directive::Directive, visitor::VisitorContext};

#[derive(Debug)]
pub struct JoinDirective {
    field_name: String,
    arguments: Vec<(Name, Value)>,
    required_fields: Vec<String>,
}

impl Directive for JoinDirective {
    fn definition() -> String {
        "directive @join(select: String!) on FIELD_DEFINITION".into()
    }
}

impl JoinDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        ctx: &mut VisitorContext<'_>,
    ) -> Option<JoinDirective> {
        let directive = directives.iter().find(|directive| directive.name.node == "join")?;

        match parse_directive::<Self>(directive, &HashMap::new()) {
            Ok(directive) => Some(directive),
            Err(error) => {
                ctx.append_errors(vec![error]);
                None
            }
        }
    }

    pub fn required_fieldset(&self) -> Option<FieldSet> {
        if self.required_fields.is_empty() {
            return None;
        }

        Some(FieldSet::new(self.required_fields.iter().map(|field| Selection {
            field: field.clone(),
            selections: vec![],
        })))
    }

    pub fn to_join_resolver(&self) -> JoinResolver {
        JoinResolver::new(self.field_name.clone(), self.arguments.clone())
    }
}

impl<'de> serde::Deserialize<'de> for JoinDirective {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct RawDirective {
            // TODO: Is this the best right name?  Not sure...
            select: String,
        }

        let raw_directive = RawDirective::deserialize(deserializer)?;

        let field = parse_field(raw_directive.select)
            .map_err(|error| D::Error::custom(format!("Could not parse join: {error}")))?;

        if !field.node.selection_set.items.is_empty() {
            return Err(D::Error::custom(
                "this join attempts to select children, but joins can only select a single field",
            ));
        }

        let arguments = field
            .node
            .arguments
            .into_iter()
            .map(|(name, value)| (name.node, value.node))
            .collect::<Vec<_>>();

        let required_fields = arguments
            .iter()
            .flat_map(|(_, value)| value.variables_used())
            .map(|variable| variable.to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        Ok(JoinDirective {
            field_name: field.node.name.node.to_string(),
            arguments,
            required_fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_json_snapshot};
    use serde::Deserialize;
    use serde_json::json;

    use crate::tests::assert_validation_error;

    use super::*;

    #[test]
    fn join_happy_path() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: String!, name: String): String! @resolver(name: "blah")
            }

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @join(select: "blah(id: $id)")
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        let resolver = &registry.types["User"].fields().as_ref().unwrap()["nickname"].resolver;

        assert_json_snapshot!(resolver, @r###"
        {
          "J": {
            "field_name": "blah",
            "arguments": [
              [
                "id",
                {
                  "$var": "id"
                }
              ]
            ]
          }
        }
        "###);
    }

    #[test]
    fn join_with_missing_required_argument() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: String!, name: String!): String! @resolver(name: "blah")
            }

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @join(select: "blah(id: $id)")
            }
            "#,
            "The field nickname of the type User is trying to join with the field named blah, but does not provide the non-nullable argument name"
        );
    }

    #[test]
    fn join_on_missing_field() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @join(select: "blah(id: $id)")
            }
            "#,
            "The field nickname of the type User is trying to join with a field named blah, which doesn't exist on the Query type"
        );
    }

    #[test]
    fn join_with_return_type_mismatch() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: String!): Int @resolver(name: "blah")
            }

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @join(select: "blah(id: $id)")
            }
            "#,
            "The field nickname of the type User is trying to join with the field named blah, but those fields do not have the same type"
        );
    }

    #[test]
    fn join_with_variable_that_doesnt_exist() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                blah(id: String!): String! @resolver(name: "blah")
            }

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @join(select: "blah(id: $whatever)")
            }
            "#,
            "The field nickname on User declares that it requires the field whatever on User but that field doesn't exist"
        );
    }

    #[test]
    fn join_directive_deser() {
        let directive = JoinDirective::deserialize(json!({"select": "findUser(name: $name, filters: {eq: $filters})"}));

        assert_debug_snapshot!(directive, @r###"
        Ok(
            JoinDirective {
                field_name: "findUser",
                arguments: [
                    (
                        Name(
                            "name",
                        ),
                        Variable(
                            Name(
                                "name",
                            ),
                        ),
                    ),
                    (
                        Name(
                            "filters",
                        ),
                        Object(
                            {
                                Name(
                                    "eq",
                                ): Variable(
                                    Name(
                                        "filters",
                                    ),
                                ),
                            },
                        ),
                    ),
                ],
                required_fields: [
                    "filters",
                    "name",
                ],
            },
        )
        "###);
    }
}
