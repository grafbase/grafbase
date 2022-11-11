use std::collections::{HashMap, HashSet};

use crate::dynamic_string::DynamicString;
use crate::rules::model_directive::MODEL_DIRECTIVE;

use super::visitor::{Visitor, VisitorContext};

use dynaql::{Positioned, ServerError};
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

use serde::{Deserialize, Serialize};

mod operations;
use operations::Operations;

const AUTH_DIRECTIVE: &str = "auth";
const DEFAULT_GROUPS_CLAIM: &str = "groups";

pub struct AuthDirective;

#[derive(Debug)]
struct Auth {
    allowed_private_ops: Operations,

    allowed_group_ops: HashMap<String, Operations>,

    allowed_owner_ops: Operations,

    providers: Vec<AuthProvider>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
enum AuthProvider {
    #[serde(rename_all = "camelCase")]
    Oidc {
        issuer: DynamicString,

        #[serde(default = "default_groups_claim")]
        groups_claim: String,
    },
}

fn default_groups_claim() -> String {
    DEFAULT_GROUPS_CLAIM.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "allow")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
enum AuthRule {
    /// Public data access via API keys
    // Ex: { allow: anonymous }
    #[serde(alias = "public")]
    #[serde(rename_all = "camelCase")]
    Anonymous {
        // Note: we don't support operations as our playground needs full access
    },

    /// Signed-in user data access via OIDC
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

    /// Owner-based data access via OIDC
    // Ex: { allow: owner }
    //     { allow: owner, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Owner {
        #[serde(default)]
        operations: Operations,
    },
}

impl AuthDirective {
    pub fn parse(
        ctx: &mut VisitorContext<'_>,
        directives: &[Positioned<ConstDirective>],
        is_global: bool,
    ) -> Result<Option<dynaql::AuthConfig>, ServerError> {
        if let Some(directive) = directives.iter().find(|d| d.node.name.node == AUTH_DIRECTIVE) {
            Auth::from_value(ctx, &directive.node, is_global).map(|auth| Some(dynaql::AuthConfig::from(auth)))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Visitor<'a> for AuthDirective {
    // This snippet is parsed, but not enforced by the server, which is why we
    // don't bother adding detailed types here.
    fn directives(&self) -> String {
        format!("directive @{AUTH_DIRECTIVE} on SCHEMA | OBJECT")
    }

    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        schema_definition: &'a dynaql::Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        match Self::parse(ctx, &schema_definition.node.directives, true) {
            Ok(Some(auth)) => {
                ctx.registry.get_mut().auth = auth;
            }
            Err(err) => {
                ctx.report_error(err.locations, err.message);
            }
            _ => {}
        }
    }

    // Visit types to check that the auth directive is used correctly. Actual
    // processing happens in the model directive.
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if let (Some(auth_directive), false) = (
            type_definition
                .node
                .directives
                .iter()
                .find(|d| d.node.name.node == AUTH_DIRECTIVE),
            type_definition
                .node
                .directives
                .iter()
                .any(|d| d.node.name.node == MODEL_DIRECTIVE),
        ) {
            ctx.report_error(
                vec![auth_directive.pos],
                format!("The @{AUTH_DIRECTIVE} directive can only be used on @{MODEL_DIRECTIVE} types"),
            );
        }
    }
}

impl Auth {
    pub fn from_value(ctx: &VisitorContext<'_>, value: &ConstDirective, is_global: bool) -> Result<Self, ServerError> {
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

        // XXX: introduce a separate type for non-global directives if we need more custom behavior
        if !is_global && !providers.is_empty() {
            return Err(ServerError::new("auth providers can only be configured globally", pos));
        }

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
                    // FIXME: replace with ::try_insert() when it's stable
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

        let allowed_owner_ops: Operations = rules
            .iter()
            .filter_map(|rule| match rule {
                AuthRule::Owner { operations, .. } => Some(operations.values().clone()),
                _ => None,
            })
            .flatten()
            .collect();

        Ok(Auth {
            allowed_private_ops,
            allowed_group_ops,
            allowed_owner_ops,
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

        let &mut AuthProvider::Oidc { ref mut issuer, .. } = &mut provider;
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
            allowed_private_ops: auth.allowed_private_ops.into(),

            allowed_group_ops: auth
                .allowed_group_ops
                .into_iter()
                .map(|(group, ops)| (group, ops.into()))
                .collect(),

            allowed_owner_ops: auth.allowed_owner_ops.into(),

            oidc_providers: auth
                .providers
                .iter()
                .map(|provider| match provider {
                    AuthProvider::Oidc { issuer, groups_claim } => dynaql::OidcProvider {
                        issuer: issuer
                            .as_fully_evaluated_str()
                            .expect(
                                "environment variables have been expanded by now \
                                and we don't support any other types of variables",
                            )
                            .parse()
                            .unwrap(),
                        groups_claim: groups_claim.clone(),
                    },
                })
                .collect(),

            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::rules::model_directive::ModelDirective;
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
                visit(&mut ModelDirective, &mut ctx, &schema);

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
                visit(&mut ModelDirective, &mut ctx, &schema);

                assert_eq!(ctx.errors.len(), 1, "errors: {:?}", ctx.errors);
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
        HashMap::from([("ISSUER_URL".to_string(), "https://my.idp.com".to_string())]),
        dynaql::AuthConfig {
            allowed_private_ops: dynaql::Operations::all(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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
        owner_rule,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: owner } ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_owner_ops: dynaql::Operations::all(),
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
            }],
            ..Default::default()
        }
    );

    parse_test!(
        owner_rule_with_ops,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
          rules: [ { allow: owner, operations: ["create"] } ],
        ){
          query: Query
        }
        "#,
        dynaql::AuthConfig {
            allowed_owner_ops: dynaql::Operations::CREATE,
            oidc_providers: vec![dynaql::OidcProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
            }],
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
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
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

    parse_fail!(
        type_auth_without_model,
        r#"
        type Todo @auth(rules: []) {
          id: ID!
        }
        "#,
        "The @auth directive can only be used on @model types"
    );

    parse_fail!(
        type_auth_with_provider,
        r#"
        type Todo @model @auth(providers: [ { type: oidc, issuer: "https://my.idp.com" } ]) {
          id: ID!
        }
        "#,
        "auth providers can only be configured globally"
    );
}
