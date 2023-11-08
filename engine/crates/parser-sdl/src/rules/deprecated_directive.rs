use engine::registry::Deprecation;
use engine_parser::{types::ConstDirective, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{directive::Directive, visitor::VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeprecatedDirective {
    pub reason: Option<String>,
}

impl DeprecatedDirective {
    pub fn from_directives(directives: &[Positioned<ConstDirective>], ctx: &mut VisitorContext<'_>) -> Deprecation {
        let Some(directive) = directives.iter().find(|directive| directive.name.node == "deprecated") else {
            return Deprecation::NoDeprecated;
        };

        match parse_directive::<Self>(directive, ctx.variables) {
            Ok(directive) => Deprecation::Deprecated {
                reason: directive.reason,
            },
            Err(error) => {
                ctx.append_errors(vec![error]);
                Deprecation::NoDeprecated
            }
        }
    }
}

impl Directive for DeprecatedDirective {
    fn definition() -> String {
        r#"
        directive @deprecated(
            reason: String = "No longer supported"
        ) on FIELD_DEFINITION | ENUM_VALUE
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {

    use crate::parse_registry;

    #[test]
    fn test_deprecated_directive() {
        let result = parse_registry(
            r#"
                extend type Query {
                    oldField: MyEnum
                        @deprecated
                        @resolver(name: "blah")
                    olderField: Int
                        @deprecated(reason: "It is older than the sun")
                        @resolver(name: "blah")
                    Blah: Int
                        @deprecated(reason: "\" there's a quote just to ruin your day")
                        @resolver(name: "blah")
                }

                enum MyEnum {
                    HAPPY_VALUE,
                    OLD_VALUE @derecated
                    OLDER_VALUE @deprecated(reason: "Dinosaurs thought this was passé")
                    QUOTED_VALUE @deprecated(reason: "\" there's a quote just to ruin your day")
                }
            "#,
        )
        .unwrap();

        insta::assert_snapshot!(result.export_sdl(false), @r###"
        enum MyEnum {
        	HAPPY_VALUE
        	OLD_VALUE
        	OLDER_VALUE @deprecated(reason: "Dinosaurs thought this was pass\u{e9}")
        	QUOTED_VALUE @deprecated(reason: "\" there\'s a quote just to ruin your day")
        }
        type Query {
        	oldField: MyEnum @deprecated
        	olderField: Int @deprecated(reason: "It is older than the sun")
        	Blah: Int @deprecated(reason: "\" there\'s a quote just to ruin your day")
        }
        schema {
        	query: Query
        }
        "###);
    }
}
