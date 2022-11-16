use dynaql::{Positioned, ServerError};
use dynaql_parser::types::ConstDirective;

use crate::rules::model_directive::MODEL_DIRECTIVE;
use crate::{Visitor, VisitorContext};

mod config;
mod operations;
mod providers;
mod rules;

use config::AuthConfig;

const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

impl AuthDirective {
    pub fn parse(
        ctx: &mut VisitorContext<'_>,
        directives: &[Positioned<ConstDirective>],
        is_global: bool,
    ) -> Result<Option<dynaql::AuthConfig>, ServerError> {
        if let Some(directive) = directives.iter().find(|d| d.node.name.node == AUTH_DIRECTIVE) {
            AuthConfig::from_value(ctx, &directive.node, is_global).map(|auth| Some(dynaql::AuthConfig::from(auth)))
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::rules::model_directive::ModelDirective;
    use crate::rules::visitor::visit;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;
    use providers::DEFAULT_GROUPS_CLAIM;

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

    parse_test!(
        issuer_url_from_variable,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{ env.ISSUER_URL }}" } ]
        ){
          query: Query
        }
        "#,
        HashMap::from([("ISSUER_URL".to_string(), "https://my.idp.com".to_string())]),
        dynaql::AuthConfig {
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
        ){
          query: Query
        }
        "#,
        HashMap::new(),
        "auth provider: right now only variables scoped with 'env.' are supported: `ISSUER_URL`"
    );

    parse_fail!(
        issuer_url_empty,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "" } ]
        ){
          query: Query
        }
        "#,
        "OIDC provider: relative URL without a base"
    );

    parse_test!(
        jwt_provider,
        r#"
        schema @auth(
          providers: [ { type: jwt, issuer: "{{ env.ISSUER_URL }}", secret: "{{ env.JWT_SECRET }}" } ]
        ){
          query: Query
        }
        "#,
        HashMap::from([
            ("ISSUER_URL".to_string(), "https://my.idp.com".to_string()),
            ("JWT_SECRET".to_string(), "s3cr3t".to_string())
        ]),
        dynaql::AuthConfig {
            jwt_providers: vec![dynaql::JwtProvider {
                issuer: url::Url::parse("https://my.idp.com").unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                secret: "s3cr3t".to_string(),
            }],
            ..Default::default()
        }
    );

    parse_fail!(
        multiple_providers,
        r#"
        schema @auth(
          providers: [ { type: foo }, { type: bar } ]
        ){
          query: Query
        }
        "#,
        "only one auth provider can be configured right now"
    );
}
