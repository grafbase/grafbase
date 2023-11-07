use engine_parser::{types::ConstDirective, Positioned};

use crate::rules::{directive::Directive, visitor::VisitorContext};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShareableDirective;

impl ShareableDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        _ctx: &mut VisitorContext<'_>,
    ) -> Option<ShareableDirective> {
        directives.iter().find(|directive| directive.name.node == "shareable")?;

        Some(ShareableDirective)
    }
}

impl Directive for ShareableDirective {
    fn definition() -> String {
        r#"
        directive @shareable on OBJECT | FIELD_DEFINITION
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn shareable_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID! @shareable
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true), @r###"
        type User @key(fields: "id" resolvable: false) {
        	id: ID! @shareable
        }
        "###);
    }

    #[test]
    fn shareable_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) @shareable {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true), @r###"
        type User @key(fields: "id" resolvable: false) @shareable {
        	id: ID!
        }
        "###);
    }
}
