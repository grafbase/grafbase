use super::{
    directive::Directive,
    model_directive::MODEL_DIRECTIVE,
    visitor::{Visitor, VisitorContext},
};
use dynaql::Positioned;
use dynaql_parser::types::{TypeDefinition, TypeKind};

pub const SEARCH_DIRECTIVE: &str = "search";

pub struct SearchDirective;

impl Directive for SearchDirective {
    fn definition() -> String {
        format!(
            r#"
            directive @{SEARCH_DIRECTIVE} on FIELD_DEFINITION
            "#
        )
    }
}

impl<'a> Visitor<'a> for SearchDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        let is_model = type_definition
            .node
            .directives
            .iter()
            .any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
        if let TypeKind::Object(object) = &type_definition.node.kind {
            for field in &object.fields {
                if let Some(directive) = field
                    .node
                    .directives
                    .iter()
                    .find(|directive| directive.node.name.node == SEARCH_DIRECTIVE)
                {
                    if !is_model {
                        ctx.report_error(
                            vec![directive.pos],
                            format!("The @{SEARCH_DIRECTIVE} directive can only be used on @{MODEL_DIRECTIVE} types."),
                        );
                    }
                }
            }
        }
    }
}
