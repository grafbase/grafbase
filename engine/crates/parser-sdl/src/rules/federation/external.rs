use engine_parser::{types::ConstDirective, Positioned};

use crate::rules::{directive::Directive, visitor::VisitorContext};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalDirective;

impl ExternalDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        _ctx: &mut VisitorContext<'_>,
    ) -> Option<ExternalDirective> {
        directives.iter().find(|directive| directive.name.node == "external")?;

        Some(ExternalDirective)
    }
}

impl Directive for ExternalDirective {
    fn definition() -> String {
        r"
        directive @external on OBJECT | FIELD_DEFINITION
        "
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn external_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID! @external
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_snapshot!(registry.export_sdl(true), @r###"
        extend schema @link(
        	url: "https://specs.apollo.dev/federation/v2.3",
        	import: ["@key", "@tag", "@shareable", "@inaccessible", "@override", "@external", "@provides", "@requires", "@composeDirective", "@interfaceObject"]
        )
        type User @key(fields: "id" resolvable: false) {
        	id: ID! @external
        }
        "###);
    }

    #[test]
    fn external_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) @external {
                id: ID!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_snapshot!(registry.export_sdl(true), @r###"
        extend schema @link(
        	url: "https://specs.apollo.dev/federation/v2.3",
        	import: ["@key", "@tag", "@shareable", "@inaccessible", "@override", "@external", "@provides", "@requires", "@composeDirective", "@interfaceObject"]
        )
        type User @key(fields: "id" resolvable: false) @external {
        	id: ID!
        }
        "###);
    }
}
