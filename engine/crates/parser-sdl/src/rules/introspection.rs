use super::{directive::Directive, visitor::Visitor};
use crate::{directive_de::parse_directive, rules::visitor::VisitorContext};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntrospectionDirective {
    pub enable: bool,
}

const INTROSPECTION_DIRECTIVE_NAME: &str = "introspection";

impl Directive for IntrospectionDirective {
    fn definition() -> String {
        r#"
        directive @introspection(
          "Whether to enable introspection."
          enabled: Boolean!,
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct IntrospectionDirectiveVisitor;

impl<'a> Visitor<'a> for IntrospectionDirectiveVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        doc: &'a engine::Positioned<engine_parser::types::SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|directive| directive.node.name.node == INTROSPECTION_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<IntrospectionDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => ctx.registry.borrow_mut().disable_introspection = !parsed_directive.enable,
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_introspection_enabled() {
        let schema = r#"
            extend schema @introspection(enable: true)
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        dbg!(&registry);

        assert!(!registry.disable_introspection);
    }

    #[test]
    fn test_introspection_disabled() {
        let schema = r#"
            extend schema @introspection(enable: false)
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        assert!(registry.disable_introspection);
    }
}
