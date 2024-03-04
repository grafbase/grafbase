use engine_parser::{types::ConstDirective, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{directive::Directive, visitor::VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TagDirective {
    name: String,
}

impl TagDirective {
    pub fn from_directives(directives: &[Positioned<ConstDirective>], ctx: &mut VisitorContext<'_>) -> Vec<String> {
        directives
            .iter()
            .filter(|directive| directive.name.node == "tag")
            .filter_map(|directive| match parse_directive::<Self>(directive, ctx.variables) {
                Ok(directive) => Some(directive.name),
                Err(error) => {
                    ctx.append_errors(vec![error]);
                    None
                }
            })
            .collect()
    }
}

impl Directive for TagDirective {
    fn definition() -> String {
        // The real tag is meant to be available in a lot more positions than this
        // but for now we're only supporting FIELD_DEFINITION position.
        //
        // These are also marked as repeatable in the actual definition but we
        // don't support that keyword at the moment
        r"
        directive @tag(name: String!) on FIELD_DEFINITION
        "
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn tag_field_on_basic_type() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @key(fields: "id", resolvable: false) {
                id: ID!
                name: Whatever! @tag(name: "woop")
                other: Whatever! @tag(name: "woop") @tag(name: "poow")
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
        	id: ID!
        	name: Whatever! @tag(name: "woop")
        	other: Whatever! @tag(name: "woop") @tag(name: "poow")
        }
        type Whatever {
        	blah: String
        }
        "###);
    }
}
