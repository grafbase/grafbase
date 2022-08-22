use std::collections::HashSet;

use super::visitor::{Visitor, VisitorContext};

use dynaql::ServerError;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

use serde::{Deserialize, Serialize};

const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

#[derive(Debug)]
struct Auth {
    allow_anonymous_access: bool,

    allow_private_access: bool,

    allowed_groups: HashSet<String>,

    providers: Vec<AuthProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
enum AuthProvider {
    #[serde(rename_all = "camelCase")]
    Oidc { issuer: url::Url },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "allow")]
#[serde(deny_unknown_fields)]
enum AuthRule {
    /// Public data access
    // Ex: { allow: anonymous }
    //     { allow: anonymous, operations: [read] }
    #[serde(alias = "public")]
    #[serde(rename_all = "camelCase")]
    Anonymous {
        #[serde(default)]
        operations: Operations,
    },

    // Signed-in user data access
    // Ex: { allow: private }
    //     { allow: private, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Private {
        #[serde(default)]
        operations: Operations,
    },

    /// User group-based data access
    // Ex: { allow: groups, groups: ["admin"] }
    #[serde(rename_all = "camelCase")]
    Groups {
        #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
        groups: HashSet<String>,

        #[serde(default)]
        operations: Operations,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Operations(#[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")] HashSet<Operation>);

impl Default for Operations {
    fn default() -> Self {
        Operations(
            vec![Operation::Create, Operation::Read, Operation::Update, Operation::Delete]
                .into_iter()
                .collect(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
enum Operation {
    Create,
    Read,
    Update,
    Delete,
    Get,  // More granual read access
    List, // More granual read access
}

impl<'a> Visitor<'a> for AuthDirective {
    // This snippet is parsed, but not enforced by the server, which is why we
    // don't bother adding detailed types here.
    fn directives(&self) -> String {
        "directive @auth on SCHEMA".to_string()
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
                    ctx.registry.get_mut().auth = auth.into();
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

        let providers = match value.get_argument("providers") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) if !value.is_empty() => value
                    .iter()
                    .map(AuthProvider::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|err| ServerError::new(err.message, pos))?,
                _ => return Err(ServerError::new("auth providers must be a non-empty list", pos)),
            },
            None => Vec::new(),
        };

        let rules = match value.get_argument("rules") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) if !value.is_empty() => value
                    .iter()
                    .map(AuthRule::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|err| ServerError::new(err.message, pos))?,
                _ => return Err(ServerError::new("auth rules must be a non-empty list", pos)),
            },
            None => Vec::new(),
        };
        dbg!(&rules);

        let allow_private_access = rules.iter().any(|rule| matches!(rule, AuthRule::Private { .. }));

        let allowed_groups: HashSet<_> = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Groups { groups, .. } => Some(groups.clone()),
                _ => None,
            })
            .flatten()
            .collect();

        if allow_private_access && !allowed_groups.is_empty() {
            return Err(ServerError::new(
                "auth rules `private` and `groups` cannot be used together",
                pos,
            ));
        }

        if providers.is_empty() {
            if allow_private_access {
                return Err(ServerError::new(
                    "auth rule `private` requires provider of type `oidc` to be configured",
                    pos,
                ));
            }
            if !allowed_groups.is_empty() {
                return Err(ServerError::new(
                    "auth rule `groups` requires provider of type `oidc` to be configured",
                    pos,
                ));
            }
        }

        Ok(Auth {
            allow_anonymous_access: true,
            allow_private_access,
            allowed_groups,
            providers,
        })
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

impl TryFrom<&ConstValue> for AuthRule {
    type Error = ServerError;

    fn try_from(value: &ConstValue) -> Result<Self, Self::Error> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth rule must be an object", None)),
        };

        let rule: AuthRule =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth rule: {err}"), None))?;

        Ok(rule)
    }
}

impl From<Auth> for dynaql::Auth {
    fn from(auth: Auth) -> Self {
        Self {
            allow_anonymous_access: auth.allow_anonymous_access,
            allow_private_access: auth.allow_private_access,
            allowed_groups: auth.allowed_groups,
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
    use super::*;
    use crate::rules::visitor::visit;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_no_auth_directive() {
        let schema = r#"
            schema {
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(ctx.registry.borrow().auth, Default::default());
    }

    #[test]
    fn test_anonymous_rule() {
        let schema = r#"
            schema @auth(
              rules: [ { allow: anonymous } ]
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(ctx.registry.borrow().auth, Default::default());
    }

    #[test]
    fn test_private_rule() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
              rules: [ { allow: private } ]
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(
            ctx.registry.borrow().auth,
            dynaql::Auth {
                allow_private_access: true,
                oidc_providers: vec![dynaql::OidcProvider {
                    issuer: url::Url::parse("https://my.idp.com").unwrap(),
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_groups_rule() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
              rules: [ { allow: groups, groups: ["admin", "moderator"] } ],
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(
            ctx.registry.borrow().auth,
            dynaql::Auth {
                allowed_groups: vec!["admin", "moderator"].into_iter().map(String::from).collect(),
                oidc_providers: vec![dynaql::OidcProvider {
                    issuer: url::Url::parse("https://my.idp.com").unwrap(),
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_groups_rule_with_duplicate_group() {
        let schema = r#"
            schema @auth(
              rules: [ { allow: groups, groups: ["A", "B", "B"] } ],
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth rule: invalid entry: found duplicate value",
        );
    }

    #[test]
    fn test_groups_rule_with_null_group() {
        let schema = r#"
            schema @auth(
              rules: [ { allow: groups, groups: ["A", null, "B"] } ],
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth rule: invalid type: null, expected a string",
        );
    }

    #[test]
    fn test_groups_rule_with_null_groups() {
        let schema = r#"
            schema @auth(
              rules: [ { allow: groups, groups: null } ],
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth rule: invalid type: null, expected a sequence",
        );
    }

    #[test]
    fn test_groups_rule_with_empty_groups() {
        let schema = r#"
            schema @auth(
              rules: [ { allow: groups, groups: [] } ],
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(ctx.registry.borrow().auth, Default::default());
    }

    #[test]
    fn test_incompatible_rules() {
        let schema = r#"
            schema @auth(
              rules: [
                { allow: groups, groups: ["admin"] },
                { allow: private }
              ]
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth rules `private` and `groups` cannot be used together",
        );
    }

    #[test]
    fn test_oidc_without_rule() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(
            ctx.registry.borrow().auth,
            dynaql::Auth {
                oidc_providers: vec![dynaql::OidcProvider {
                    issuer: url::Url::parse("https://my.idp.com").unwrap(),
                }],
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_oidc_with_missing_field() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc } ]
            ){
              query: Query
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth provider: missing field `issuer`",
        );
    }
}
