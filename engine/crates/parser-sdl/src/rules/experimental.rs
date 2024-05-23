use std::collections::HashMap;

use engine_parser::{types::SchemaDefinition, Positioned};

use crate::{
    directive_de::parse_directive,
    rules::{
        directive::Directive,
        visitor::{RuleError, Visitor, VisitorContext},
    },
};

use super::visitor::Warning;

const EXPERIMENTAL_DIRECTIVE_NAME: &str = "experimental";

#[derive(Debug, thiserror::Error)]
pub enum ExperimentalDirectiveError {
    #[error("Unable to parse @experimental - {0}")]
    Parsing(RuleError),
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExperimentalDirective {
    /// These three are all removed or no longer experimental.
    ///
    /// For now I've added warnings if users are still specifying this, but we should remove eventually.
    pub kv: Option<bool>,
    pub ai: Option<bool>,
    pub codegen: Option<bool>,

    pub partial_caching: Option<bool>,
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
                if experimental_directive.kv.is_some() {
                    ctx.warnings.push(Warning::ExperimentalFeatureRemoved("kv".into()));
                }
                if experimental_directive.ai.is_some() {
                    ctx.warnings.push(Warning::ExperimentalFeatureRemoved("ai".into()));
                }

                if let Some(enabled) = experimental_directive.codegen {
                    ctx.warnings.push(Warning::ExperimentalFeaturePromoted {
                        feature: "codegen".into(),
                        documentation: "https://grafbase.com/docs/resolvers/codegen".into(),
                    });
                    ctx.registry.get_mut().codegen = Some(registry_v2::CodegenConfig { enabled, path: None });
                }

                if let Some(enabled) = experimental_directive.partial_caching {
                    ctx.warnings.push(Warning::ExperimentalFeatureUnreleased {
                        feature: "partial_caching".into(),
                    });
                    ctx.registry.get_mut().enable_partial_caching = enabled;
                }
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
    ", &["Unable to parse @experimental - [2:37] unknown field `random`, expected one of `kv`, `ai`, `codegen`, `partialCaching`"], "", false)]
    #[case::successful_parsing_codegen_disabled(r"
        extend schema @experimental(codegen: false)
    ", &[], "codegen", false)]
    #[case::successful_parsing_codegen_enabled(r"
        extend schema @experimental(codegen: true)
    ", &[], "codegen", true)]
    #[case::successful_parsing_partial_caching_enabled(r"
        extend schema @experimental(partialCaching: true)
    ", &[], "partialCaching", true)]
    #[case::successful_parsing_partial_caching_disabled(r"
        extend schema @experimental(partialCaching: false)
    ", &[], "partialCaching", false)]
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
            "codegen" => assert_eq!(ctx.registry.borrow().codegen.as_ref().unwrap().enabled, expected),
            "partialCaching" => assert_eq!(ctx.registry.borrow().enable_partial_caching, expected),
            _ => {}
        }
    }
}
