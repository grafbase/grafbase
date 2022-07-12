use super::visitor::{Visitor, VisitorContext};

pub struct AuthDirective;

pub const AUTH_DIRECTIVE: &str = "auth";

impl<'a> Visitor<'a> for AuthDirective {
    // FIXME: this snippet is not enforced by the server
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
            match (&directive.node).try_into() {
                Ok(auth) => ctx.registry.get_mut().auth = Some(auth),
                Err(err) => ctx.report_error(vec![directive.pos], err.message),
            }
        }
    }
}
