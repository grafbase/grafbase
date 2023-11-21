use std::collections::HashMap;

use engine_parser::{types::ConstDirective, Positioned};

use super::{federation::FieldSet, visitor::VisitorContext};
use crate::{directive_de::parse_directive, rules::directive::Directive};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequiresDirective {
    fields: FieldSet,
}

impl RequiresDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        ctx: &mut VisitorContext<'_>,
    ) -> Option<RequiresDirective> {
        let directive = directives.iter().find(|directive| directive.name.node == "requires")?;

        match parse_directive::<Self>(directive, &HashMap::new()) {
            Ok(directive) => Some(directive),
            Err(error) => {
                ctx.append_errors(vec![error]);
                None
            }
        }
    }

    pub fn into_fields(self) -> engine::registry::FieldSet {
        self.fields.0
    }
}

impl Directive for RequiresDirective {
    fn definition() -> String {
        r"
        directive @requires(fields: FieldSet!) on FIELD_DEFINITION
        "
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::assert_validation_error;

    #[test]
    fn requires_on_normal_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                name: String!
                account: Account!
                field: String! @requires(fields: "id name account { id }")
            }

            type Account {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        let requires = registry.types["User"].fields().as_ref().unwrap()["field"]
            .requires
            .as_ref()
            .unwrap();

        assert_eq!(requires.to_string(), "id name account { id }");
    }

    #[test]
    fn require_a_missing_field_on_current_type() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                nickname: String! @requires(fields: "id name")
            }
            "#,
            "The field nickname on the type User declares that it requires the field name on User but that field doesn't exist"
        );
    }

    #[test]
    fn require_a_missing_field_on_nested_type() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                account: Account!
                nickname: String! @requires(fields: "id account { name }")
            }

            type Account {
                id: ID!
            }
            "#,
            "The field nickname on the type User declares that it requires the field name on Account but that field doesn't exist"
        );
    }

    #[test]
    fn require_subfields_of_leaf_type() {
        assert_validation_error!(
            r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                name: String!
                nickname: String! @requires(fields: "id name { blah }")
            }
            "#,
            "The field nickname on the type User tries to require subfields of name on User but that field is a leaf type"
        );
    }
}
