use engine_parser::{types::ConstDirective, Positioned};

use crate::rules::{directive::Directive, visitor::VisitorContext};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InaccessibleDirective;

impl InaccessibleDirective {
    pub fn from_directives(directives: &[Positioned<ConstDirective>], _ctx: &mut VisitorContext<'_>) -> bool {
        directives.iter().any(|directive| directive.name.node == "inaccessible")
    }
}

impl Directive for InaccessibleDirective {
    fn definition() -> String {
        // The real inaccessible is meant to be available in a lot more positions than this
        // but for now we're only supporting this one
        r#"
        directive @inaccessible on FIELD_DEFINITION
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn inaccessible_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID! @inaccessible
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true), @r###"
        type User @key(fields: "id" resolvable: false) {
        	id: ID! @inaccessible
        }
        "###);
    }
}
