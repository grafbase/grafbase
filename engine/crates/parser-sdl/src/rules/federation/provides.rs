use engine_parser::{types::ConstDirective, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{directive::Directive, visitor::VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvidesDirective {
    // Technically fields is a FieldSet, but we don't really use it so
    // I'm just keeping it as a String
    pub fields: String,
}

impl ProvidesDirective {
    pub fn from_directives(
        directives: &[Positioned<ConstDirective>],
        ctx: &mut VisitorContext<'_>,
    ) -> Option<ProvidesDirective> {
        let directive = directives.iter().find(|directive| directive.name.node == "provides")?;

        match parse_directive::<Self>(directive, ctx.variables) {
            Ok(directive) => Some(directive),
            Err(error) => {
                ctx.append_errors(vec![error]);
                None
            }
        }
    }
}

impl Directive for ProvidesDirective {
    fn definition() -> String {
        r"
        directive @provides(fields: FieldSet!) on FIELD_DEFINITION
        "
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn provides_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID! @external
                name: Whatever! @provides(fields: "blah")
            }

            type Whatever {
                blah: String
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
        	name: Whatever! @provides(fields: "blah")
        }
        type Whatever {
        	blah: String
        }
        "###);
    }
}
