use std::collections::{HashMap, HashSet};

use crate::dynamic_string::DynamicString;

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

    allowed_group_ops: HashMap<String, Operations>,

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
    Oidc { issuer: DynamicString },
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
            match Auth::from_value(ctx, &directive.node) {
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

impl Auth {
    fn from_value(ctx: &VisitorContext<'_>, value: &ConstDirective) -> Result<Self, ServerError> {
        let pos = Some(value.name.pos);

        let providers = match value.get_argument("providers") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) if !value.is_empty() => value
                    .iter()
                    .map(|value| AuthProvider::from_value(ctx, value))
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
                    .map(|value| AuthRule::from_value(ctx, value))
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

        let allowed_group_ops = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Groups { groups, operations } => Some((groups, operations)),
                _ => None,
            })
            .try_fold(HashMap::new(), |mut res, (groups, operations)| {
                if groups.is_empty() {
                    return Err(ServerError::new("groups must be a non-empty list", pos));
                }
                for group in groups {
                    if res.contains_key(group) {
                        return Err(ServerError::new(
                            format!("group {group:?} cannot be used in more than one auth rule"),
                            pos,
                        ));
                    }
                    res.insert(group.clone(), operations.clone());
                }
                Ok(res)
            })?;

        if providers.is_empty() {
            if allowed_private_ops.any() {
                return Err(ServerError::new(
                    "auth rule `private` requires provider of type `oidc` to be configured",
                    pos,
                ));
            }
            if !allowed_group_ops.is_empty() {
                return Err(ServerError::new(
                    "auth rule `groups` requires provider of type `oidc` to be configured",
                    pos,
                ));
            }
        }

        Ok(Auth {
            allowed_anonymous_ops: Operations::default(),
            allowed_private_ops,
            allowed_group_ops,
            providers,
        })
    }
}

impl AuthProvider {
    fn from_value(ctx: &VisitorContext<'_>, value: &ConstValue) -> Result<Self, ServerError> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let mut provider: AuthProvider =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth provider: {err}"), None))?;

        let &mut AuthProvider::Oidc { ref mut issuer } = &mut provider;
        ctx.partially_evaluate_literal(issuer)?;
        if let Err(err) = issuer
            .as_fully_evaluated_str()
            .map(|s| s.parse::<url::Url>())
            .transpose()
        {
            // FIXME: Pass in the proper location here and everywhere above as it's not done properly now.
            return Err(ServerError::new(format!("auth provider: {err}"), None));
        }

        Ok(provider)
    }
}

impl AuthRule {
    fn from_value(_ctx: &VisitorContext<'_>, value: &ConstValue) -> Result<Self, ServerError> {
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

            allowed_group_ops: auth
                .allowed_group_ops
                .into_iter()
                .map(|(group, ops)| (group, ops.into()))
                .collect(),

            oidc_providers: auth
                .providers
                .iter()
                .map(|provider| match provider {
                    AuthProvider::Oidc { issuer } => dynaql::OidcProvider {
                        issuer: issuer
                            .as_fully_evaluated_str()
                            .expect(
                                "environment variables have been expanded by now \
                                and we don't support any other types of variables",
                            )
                            .parse()
                            .unwrap(),
                    },
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::rules::visitor::visit;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;

    macro_rules! parse_test {
        ($fn_name:ident, $schema:literal, $expect:expr) => {
            parse_test!($fn_name, $schema, HashMap::new(), $expect);
        };
        ($fn_name:ident, $schema:literal, $variables:expr, $expect:expr) => {
            #[test]
            fn $fn_name() {
                let variables = $variables;
                let schema = parse_schema($schema).unwrap();
                let mut ctx = VisitorContext::new_with_variables(&schema, &variables);
                visit(&mut AuthDirective, &mut ctx, &schema);

                assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);
                assert_eq!(ctx.registry.borrow().auth, $expect);
            }
        };
    }

    macro_rules! parse_fail {
        ($fn_name:ident, $schema:literal, $err:literal) => {
            parse_fail!($fn_name, $schema, HashMap::new(), $err);
        };
        ($fn_name:ident, $schema:literal, $variables:expr, $err:literal) => {
            #[test]
            fn $fn_name() {
                let variables = $variables;
                let schema = parse_schema($schema).unwrap();
                let mut ctx = VisitorContext::new_with_variables(&schema, &variables);
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
        issuer_url_from_variable,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{ env.ISSUER_URL }}" } ]
          rules: [ { allow: private } ]
        ){
          query: Query
        }
        "#,
        HashMap::from([("ISSUER_URL".to_string(), "https://my.idp.com".to_string()),]),
        dynaql::AuthConfig {
            allowed_private_ops: dynaql::Operations::all(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
            }],
            ..Default::default()
        }
    );

    parse_fail!(
        issuer_url_from_nonexistent_variable,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{ env.ISSUER_URL }}" } ]
          rules: [ { allow: private } ]
        ){
          query: Query
        }
        "#,
        HashMap::new(),
        "undefined variable `ISSUER_URL`"
    );

    parse_fail!(
        issuer_url_from_invalid_template_key,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{ ISSUER_URL }}" } ]
          rules: [ { allow: private } ]
        ){
          query: Query
        }
        "#,
        HashMap::new(),
        "auth provider: right now only variables scoped with 'env.' are supported: `ISSUER_URL`"
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
            allowed_group_ops: HashMap::from_iter(vec![
                ("admin".to_string(), dynaql::Operations::all()),
                ("moderator".to_string(), dynaql::Operations::all()),
            ]),
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
          rules: [
            { allow: groups, groups: ["admin"] }
            { allow: groups, groups: ["moderator", "editor"], operations: ["get", "list"] }
          ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_group_ops: HashMap::from_iter(vec![
                ("admin".to_string(), dynaql::Operations::all()),
                (
                    "moderator".to_string(),
                    dynaql::Operations::GET | dynaql::Operations::LIST
                ),
                ("editor".to_string(), dynaql::Operations::GET | dynaql::Operations::LIST)
            ]),
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

    parse_fail!(
        groups_rule_with_empty_groups,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: [] } ],
        ){
          query: Query
        }
        "#,
        "groups must be a non-empty list"
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
