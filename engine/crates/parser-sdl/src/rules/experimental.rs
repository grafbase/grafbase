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
    #[error("Unable to parse @experimental - {0}")]
    Parsing(RuleError),
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalDirective {
    pub kv: Option<bool>,
    pub ai: Option<bool>,
    pub codegen: Option<bool>,
}

pub struct ExperimentalDirectiveVisitor;
impl<'a> Visitor<'a> for ExperimentalDirectiveVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        let Some(experimental_directive) = doc
            .directives
            .iter()
            .find(|d| d.node.name.node == EXPERIMENTAL_DIRECTIVE_NAME)
        else {
            return;
        };

        match parse_directive::<ExperimentalDirective>(experimental_directive, &HashMap::default()) {
            Ok(experimental_directive) => {
                ctx.registry.get_mut().enable_kv = experimental_directive.kv.unwrap_or_default();
                ctx.registry.get_mut().enable_ai = experimental_directive.ai.unwrap_or_default();
                ctx.registry.get_mut().enable_codegen = experimental_directive.codegen.unwrap_or_default();
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
          """
          Enable experimental usage of AI in resolvers.
          """
          ai: Boolean
          """
          Enable experimental typed resolver code generation.
          """
          codegen: Boolean
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
    #[case::error_parsing_unknown_field(r"
        extend schema @experimental(random: true)
    ", &["Unable to parse @experimental - [2:37] unknown field `random`, expected one of `kv`, `ai`, `codegen`"], "", false)]
    #[case::successful_parsing_kv_enabled(r"
        extend schema @experimental(kv: true)
    ", &[], "kv", true)]
    #[case::successful_parsing_kv_disabled(r"
        extend schema @experimental(kv: false)
    ", &[], "kv", false)]
    #[case::successful_parsing_ai_enabled(r"
        extend schema @experimental(ai: true)
    ", &[], "ai", true)]
    #[case::successful_parsing_ai_disabled(r"
        extend schema @experimental(ai: false)
    ", &[], "ai", false)]
    #[case::successful_parsing_codegen_disabled(r"
        extend schema @experimental(codegen: false)
    ", &[], "codegen", false)]
    #[case::successful_parsing_codegen_enabled(r"
        extend schema @experimental(codegen: true)
    ", &[], "codegen", true)]
    fn test_parsing(
        #[case] schema: &str,
        #[case] expected_messages: &[&str],
        #[case] target: &str,
        #[case] expected: bool,
    ) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut ExperimentalDirectiveVisitor, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);

        match target {
            "ai" => assert_eq!(ctx.registry.borrow().enable_ai, expected),
            "kv" => assert_eq!(ctx.registry.borrow().enable_kv, expected),
            "codegen" => assert_eq!(ctx.registry.borrow().enable_codegen, expected),
            _ => {}
        }
    }
}
