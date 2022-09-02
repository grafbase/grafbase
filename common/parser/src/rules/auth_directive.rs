use std::collections::HashSet;

use super::visitor::{Visitor, VisitorContext};

use dynaql::ServerError;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

#[derive(Debug)]
struct Auth {
    allowed_anonymous_ops: Operations,

    allowed_private_ops: Operations,

    allowed_groups: HashSet<String>,
    allowed_group_ops: Operations,

    providers: Vec<AuthProvider>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Operations(#[serde(with = "sets_duplicate_value_is_error")] HashSet<Operation>);

impl std::iter::FromIterator<Operation> for Operations {
    fn from_iter<I: IntoIterator<Item = Operation>>(iter: I) -> Self {
        Operations(iter.into_iter().collect())
    }
}

impl Default for Operations {
    fn default() -> Self {
        [Operation::Create, Operation::Read, Operation::Update, Operation::Delete]
            .into_iter()
            .collect()
    }
}

impl Operations {
    fn values(&self) -> &HashSet<Operation> {
        &self.0
    }

    fn any(&self) -> bool {
        !self.0.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "camelCase")]
enum Operation {
    Create,
    Read,
    Get,  // More granual read access
    List, // More granual read access
    Update,
    Delete,
}

impl From<Operations> for dynaql::Operations {
    fn from(ops: Operations) -> Self {
        let mut res = Self::empty();
        for op in ops.values() {
            res |= match op {
                Operation::Create => Self::CREATE,
                Operation::Read => Self::READ,
                Operation::Get => Self::GET,
                Operation::List => Self::LIST,
                Operation::Update => Self::UPDATE,
                Operation::Delete => Self::DELETE,
            };
        }
        res
    }
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
    /// Public data access via API keys
    // Ex: { allow: anonymous }
    #[serde(alias = "public")]
    #[serde(rename_all = "camelCase")]
    Anonymous {
        // Note: we don't support operations as our playground needs full access
    },

    // Signed-in user data access via OIDC
    // Ex: { allow: private }
    //     { allow: private, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Private {
        #[serde(default)]
        operations: Operations,
    },

    /// User group-based data access via OIDC
    // Ex: { allow: groups, groups: ["admin"] }
    //     { allow: groups, groups: ["admin"], operations: [update, delete] }
    #[serde(rename_all = "camelCase")]
    Groups {
        #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
        groups: HashSet<String>,

        #[serde(default)]
        operations: Operations,
    },
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

        let allowed_private_ops: Operations = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Private { operations, .. } => Some(operations.values().clone()),
                _ => None,
            })
            .flatten()
            .collect();

        let allowed_group_ops: Operations = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Groups { operations, .. } => Some(operations.values().clone()),
                _ => None,
            })
            .flatten()
            .collect();

        // TODO: don't merge all groups, but handle ops per group
        let allowed_groups: HashSet<_> = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Groups { groups, .. } => Some(groups.clone()),
                _ => None,
            })
            .flatten()
            .collect();

        // TODO: this should be possible
        if allowed_private_ops.any() && allowed_group_ops.any() {
            return Err(ServerError::new(
                "auth rules `private` and `groups` cannot be used together",
                pos,
            ));
        }

        if providers.is_empty() {
            if allowed_private_ops.any() {
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
            allowed_anonymous_ops: Operations::default(),
            allowed_private_ops,
            allowed_groups,
            allowed_group_ops,
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

impl From<Auth> for dynaql::AuthConfig {
    fn from(auth: Auth) -> Self {
        Self {
            allowed_anonymous_ops: auth.allowed_anonymous_ops.into(),

            allowed_private_ops: auth.allowed_private_ops.into(),

            allowed_groups: auth.allowed_groups,
            allowed_group_ops: auth.allowed_group_ops.into(),

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

    macro_rules! parse_test {
        ($fn_name:ident, $schema:literal, $expect:expr) => {
            #[test]
            fn $fn_name() {
                let schema = parse_schema($schema).unwrap();
                let mut ctx = VisitorContext::new(&schema);
                visit(&mut AuthDirective, &mut ctx, &schema);

                assert!(ctx.errors.is_empty());
                assert_eq!(ctx.registry.borrow().auth, $expect);
            }
        };
    }

    macro_rules! parse_fail {
        ($fn_name:ident, $schema:literal, $err:literal) => {
            #[test]
            fn $fn_name() {
                let schema = parse_schema($schema).unwrap();
                let mut ctx = VisitorContext::new(&schema);
                visit(&mut AuthDirective, &mut ctx, &schema);

                assert_eq!(ctx.errors.len(), 1);
                assert_eq!(ctx.errors.get(0).unwrap().message, $err);
            }
        };
    }

    parse_test!(
        no_auth_directive,
        r#"
        schema {
          query: Query
        }
        "#,
        Default::default()
    );

    parse_test!(
        anonymous_rule,
        r#"
        schema @auth(
          rules: [ { allow: anonymous } ]
        ){
          query: Query
        }
        "#,
        Default::default()
    );

    parse_fail!(
        anonymous_rule_with_unsupported_ops,
        r#"
        schema @auth(
          rules: [ { allow: anonymous, operations: [read] } ]
        ){
          query: Query
        }
        "#,
        "auth rule: unknown field `operations`, there are no fields"
    );

    parse_test!(
        private_rule,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: private } ]
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_private_ops: dynaql::Operations::all(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_test!(
        private_rule_with_ops,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: private, operations: [create, delete] } ]
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_private_ops: dynaql::Operations::CREATE | dynaql::Operations::DELETE,
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_test!(
        private_rule_with_empty_ops,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: private, operations: [] } ]
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_private_ops: dynaql::Operations::empty(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_fail!(
        incompatible_rules,
        r#"
        schema @auth(
          rules: [
            { allow: groups, groups: ["admin"] },
            { allow: private }
          ]
        ){
          query: Query
        }
        "#,
        "auth rules `private` and `groups` cannot be used together"
    );

    parse_test!(
        groups_rule,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: groups, groups: ["admin", "moderator"] } ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_groups: vec!["admin", "moderator"].into_iter().map(String::from).collect(),
            allowed_group_ops: dynaql::Operations::all(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_test!(
        groups_rule_with_ops,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: groups, groups: ["admin", "moderator"], operations: ["get"] } ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_groups: vec!["admin", "moderator"].into_iter().map(String::from).collect(),
            allowed_group_ops: dynaql::Operations::GET,
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_fail!(
        groups_rule_with_duplicate_group,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: ["A", "B", "B"] } ],
        ){
          query: Query
        }
        "#,
        "auth rule: invalid entry: found duplicate value"
    );

    parse_fail!(
        groups_rule_with_null_group,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: ["A", null, "B"] } ],
        ){
          query: Query
        }
        "#,
        "auth rule: invalid type: null, expected a string"
    );

    parse_fail!(
        groups_rule_with_null_groups,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: null } ],
        ){
          query: Query
        }
        "#,
        "auth rule: invalid type: null, expected a sequence"
    );

    parse_test!(
        groups_rule_with_empty_groups,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: [] } ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_group_ops: dynaql::Operations::all(),
            ..Default::default()
        }
    );

    parse_test!(
        oidc_without_rule,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_fail!(
        oidc_with_missing_field,
        r#"
        schema @auth(
          providers: [ { type: oidc } ]
        ){
          query: Query
        }
        "#,
        "auth provider: missing field `issuer`"
    );
}
