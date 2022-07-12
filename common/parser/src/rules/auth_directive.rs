use super::visitor::{Visitor, VisitorContext};

pub struct AuthDirective;

pub const AUTH_DIRECTIVE: &str = "auth";

impl<'a> Visitor<'a> for AuthDirective {
    // FIXME: this snippet is parsed, but not enforced by the server
    fn directives(&self) -> String {
        r#"
        directive @auth(providers: [AuthProviderDefinition!]!) on SCHEMA
        input AuthProviderDefinition {
          type: String!
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

#[cfg(test)]
mod tests {
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    #[test]
    fn test_oidc_ok() {
        let schema = r#"
            schema @auth(providers: [
              { type: "oidc", issuer: "https://clerk.b74v0.5y6hj.lcl.dev" }
            ]) {
              query: Boolean # HACK: make top-level auth directive work
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut super::AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
    }

    #[test]
    fn test_oidc_missing_issuer() {
        let schema = r#"
            schema @auth(providers: [
              { type: "oidc" }
            ]) {
              query: Boolean
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut super::AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(ctx.errors.get(0).unwrap().message, "auth provider: issuer missing",);
    }
}
