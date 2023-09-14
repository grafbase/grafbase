use std::collections::HashMap;

use engine_parser::{types::SchemaDefinition, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{
        directive::Directive,
        visitor::{RuleError, Visitor, VisitorContext},
    },
};

const EXPERIMENTAL_DIRECTIVE_NAME: &str = "experimental";

#[derive(Debug, thiserror::Error)]
pub enum ExperimentalDirectiveError {
    #[error("Unable to parse @experiment - {0}")]
    Parsing(RuleError),
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalDirective {
    pub kv: bool,
}

pub struct ExperimentalDirectiveVisitor;
impl<'a> Visitor<'a> for ExperimentalDirectiveVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        let directive = doc
            .directives
            .iter()
            .find(|d| d.node.name.node == EXPERIMENTAL_DIRECTIVE_NAME);

        let Some(experimental_directive) = directive else {
            return;
        };

        match parse_directive::<ExperimentalDirective>(experimental_directive, &HashMap::default()) {
            Ok(experimental_directive) => {
                ctx.registry.get_mut().enable_kv = experimental_directive.kv;
            }
            Err(err) => {
                ctx.report_error(
                    vec![experimental_directive.pos],
                    ExperimentalDirectiveError::Parsing(err).to_string(),
                );
            }
        };
    }
}

impl Directive for ExperimentalDirective {
    fn definition() -> String {
        r#"
        directive @experimental(
          """
          Enable experimental usage of KV in resolvers.
          """
          kv: Boolean
        ) on SCHEMA
        "#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parse_schema,
        rules::{
            experimental::ExperimentalDirectiveVisitor,
            visitor::{visit, VisitorContext},
        },
    };

    #[rstest::rstest]
    #[case::error_parsing_unknown_field(r#"
        extend schema @experimental(random: true)
    "#, &["@experimental error: Unable to parse - [2:37] unknown field `random`, expected `kv`"], false)]
    #[case::successful_parsing_kv_enabled(r#"
        extend schema @experimental(kv: true)
    "#, &[], true)]
    #[case::successful_parsing_kv_disabled(r#"
        extend schema @experimental(kv: false)
    "#, &[], false)]
    fn test_parsing(#[case] schema: &str, #[case] expected_messages: &[&str], #[case] expected_kv_enabled: bool) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut ExperimentalDirectiveVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);

        assert_eq!(ctx.registry.borrow().enable_kv, expected_kv_enabled);
    }
}
