use super::visitor::{Visitor, VisitorContext};

pub struct AuthDirective;

pub const AUTH_DIRECTIVE: &str = "auth";

impl<'a> Visitor<'a> for AuthDirective {
    fn directives(&self) -> String {
        r#"
        directive @auth(providers: [AuthProviderDefinition!]!) on SCHEMA
        input AuthProviderDefinition {
          issuer: String
        }
        "#
        .to_string()
    }

    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if let Some(directive) = &type_definition
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == AUTH_DIRECTIVE)
        {
            ctx.registry.get_mut().auth = Some(directive.node.clone().try_into().unwrap());
        }
    }
}
