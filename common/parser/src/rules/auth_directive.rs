use super::visitor::{Visitor, VisitorContext};

pub struct AuthDirective;

pub const AUTH_DIRECTIVE: &str = "auth";

impl<'a> Visitor<'a> for AuthDirective {
    fn directives(&self) -> String {
        r#"
        directive @auth(providers: [AuthProviderDefinition!]!) on SCHEMA
        input AuthProviderDefinition {
          issuer: String!
        }
        "#
        .to_string()
    }

    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        schema_definition: &'a dynaql::Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        if let Some(directive) = schema_definition
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == AUTH_DIRECTIVE)
        {
            // TODO: use ctx.report_error
            ctx.registry.get_mut().auth = Some(directive.node.clone().try_into().unwrap());
        }
    }
}
