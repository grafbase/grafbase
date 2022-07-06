use super::visitor::{Visitor, VisitorContext};
use dynaql::indexmap::IndexMap;
use dynaql::model::__DirectiveLocation;
use dynaql::registry::MetaDirective;

pub struct AuthDirective;

pub const AUTH_DIRECTIVE: &str = "auth";

impl<'a> Visitor<'a> for AuthDirective {
    fn directives(&self) -> String {
        format!("directive @{AUTH_DIRECTIVE} on SCHEMA")
    }

    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        let directive = &type_definition
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == AUTH_DIRECTIVE);

        if directive.is_some() {
            ctx.registry.get_mut().add_directive(MetaDirective {
                name: AUTH_DIRECTIVE.to_string(),
                description: None,
                locations: vec![__DirectiveLocation::SCHEMA],
                args: {
                    // TODO: parse directive
                    IndexMap::new()
                },
                is_repeatable: false,
                visible: None,
            });
        }
    }
}
