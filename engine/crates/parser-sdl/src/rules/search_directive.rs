use engine::Positioned;
use engine_parser::types::TypeDefinition;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

pub const SEARCH_DIRECTIVE: &str = "search";

pub struct SearchDirective;

impl Directive for SearchDirective {
    fn definition() -> String {
        format!(
            r#"
            directive @{SEARCH_DIRECTIVE} on OBJECT | FIELD_DEFINITION
            "#
        )
    }
}

impl<'a> Visitor<'a> for SearchDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        if !type_definition
            .node
            .directives
            .iter()
            .any(|directive| directive.is_search())
        {
            return;
        }

        ctx.report_error(
            vec![type_definition.node.name.pos],
            "The `@search` directive is no longer supported.",
        );
    }
}
