use super::visitor::{Visitor, VisitorContext};

use dynaql::ServerError;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub providers: Vec<AuthProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum AuthProvider {
    Oidc { issuer: url::Url },
}

impl<'a> Visitor<'a> for AuthDirective {
    // FIXME: this snippet is parsed, but not enforced by the server, why?
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
            match (&directive.node).try_into() as Result<Auth, ServerError> {
                Ok(auth) => {
                    ctx.registry.get_mut().auth = Some(auth.into());
                }
                Err(err) => {
                    ctx.report_error(vec![directive.pos], err.message);
                }
            }
        }
    }
}

impl TryFrom<&ConstDirective> for Auth {
    type Error = ServerError;

    fn try_from(value: &ConstDirective) -> Result<Self, Self::Error> {
        let pos = Some(value.name.pos);

        let arg = match value.get_argument("providers") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) => value,
                _ => return Err(ServerError::new("auth providers must be a list", pos)),
            },
            None => return Err(ServerError::new("auth providers missing", pos)),
        };

        let providers = arg
            .iter()
            .map(AuthProvider::try_from)
            .collect::<Result<_, _>>()
            .map_err(|err| ServerError::new(err.message, pos))?;

        Ok(Auth { providers })
    }
}

impl TryFrom<&ConstValue> for AuthProvider {
    type Error = ServerError;

    fn try_from(value: &ConstValue) -> Result<Self, Self::Error> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let provider: AuthProvider =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth provider: {err}"), None))?;

        Ok(provider)
    }
}

impl From<Auth> for dynaql::Auth {
    fn from(auth: Auth) -> Self {
        Self {
            oidc_providers: auth
                .providers
                .iter()
                .map(|provider| match provider {
                    AuthProvider::Oidc { issuer } => dynaql::OidcProvider { issuer: issuer.clone() },
                })
                .collect(),
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
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth provider: missing field `issuer`",
        );
    }
}
