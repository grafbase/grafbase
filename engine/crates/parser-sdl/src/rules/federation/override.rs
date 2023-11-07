use engine_parser::{types::ConstDirective, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{directive::Directive, visitor::VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OverrideDirective {
    pub from: String,
}

impl OverrideDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        ctx: &mut VisitorContext<'_>,
    ) -> Option<OverrideDirective> {
        let directive = directives.iter().find(|directive| directive.name.node == "override")?;

        match parse_directive::<Self>(directive, ctx.variables) {
            Ok(directive) => Some(directive),
            Err(error) => {
                ctx.append_errors(vec![error]);
                None
            }
        }
    }
}

impl Directive for OverrideDirective {
    fn definition() -> String {
        r#"
        directive @override(from: String!) on FIELD_DEFINITION
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn override_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID! @external
                name: String! @override(from: "Accounts")
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true), @r###"
        type User @key(fields: "id" resolvable: false) {
        	id: ID! @external
        	name: String! @override(from: "Accounts")
        }
        "###);
    }
}
