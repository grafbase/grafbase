use std::collections::HashMap;

use engine_parser::{types::ConstDirective, Positioned};

use crate::{directive_de::parse_directive, rules::directive::Directive};

use super::{federation::FieldSet, visitor::VisitorContext};

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
        r#"
        directive @requires(fields: FieldSet!) on FIELD_DEFINITION
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn requires_on_normal_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                name: String!
                field: String! @requires(fields: "id name")
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        let requires = registry.types["User"].fields().as_ref().unwrap()["field"]
            .requires
            .as_ref()
            .unwrap();

        assert_eq!(requires.to_string(), "id name");
    }
}
