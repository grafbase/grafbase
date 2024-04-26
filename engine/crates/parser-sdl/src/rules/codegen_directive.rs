use super::{directive::Directive, visitor::Visitor};
use crate::directive_de::parse_directive;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodegenDirective {
    pub enabled: bool,
    /// The target file path for the generated code.
    pub path: Option<String>,
}

impl Directive for CodegenDirective {
    fn definition() -> String {
        r#"
            directive @codegen(
                enabled: Boolean! = true
                """The target file path for the generated code."""
                path: String
            ) on SCHEMA
            
        "#
        .to_owned()
    }
}

const CODEGEN_DIRECTIVE_NAME: &str = "codegen";

pub struct CodegenVisitor;

impl<'a> Visitor<'a> for CodegenVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut super::visitor::VisitorContext<'a>,
        doc: &'a engine::Positioned<engine_parser::types::SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == CODEGEN_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<CodegenDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    ctx.codegen_directive = Some(parsed_directive);
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    fn parse_schema(schema: &str) -> crate::ParseResult<'_> {
        let connector_parsers = crate::connector_parsers::MockConnectorParsers::default();
        let variables = HashMap::default();
        futures::executor::block_on(crate::parse(&schema, &variables, &connector_parsers)).unwrap()
    }

    #[test]
    fn test_codegen_directive_enabled_with_path() {
        let schema = r#"
            extend schema @codegen(enabled: true, path: "/dev/null")
        "#;
        let parsed = parse_schema(schema);

        let config = parsed.registry.codegen.unwrap();
        assert!(config.enabled);
        assert_eq!(config.path.as_deref(), Some("/dev/null"));
    }

    #[test]
    fn test_codegen_directive_disabled() {
        let schema = r#"
            extend schema @codegen(enabled: false)
        "#;
        let parsed = parse_schema(schema);

        let config = parsed.registry.codegen.unwrap();
        assert!(!config.enabled);
        assert!(config.path.is_none());
    }
}
