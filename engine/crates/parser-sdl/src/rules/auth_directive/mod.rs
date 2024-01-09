use engine::{Positioned, ServerError};
use engine_parser::types::{ConstDirective, FieldDefinition, SchemaDefinition, TypeDefinition};

use crate::{rules::model_directive::MODEL_DIRECTIVE, Visitor, VisitorContext};

mod config;
mod operations;
mod providers;
mod rules;

use super::directive::Directive;

const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

impl AuthDirective {
    pub fn parse(
        ctx: &VisitorContext<'_>,
        directives: &[Positioned<ConstDirective>],
        is_global: bool,
    ) -> Result<Option<engine::AuthConfig>, ServerError> {
        if let Some(directive) = directives.iter().find(|d| d.is_auth()) {
            config::parse_auth_config(ctx, directive, is_global).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl Directive for AuthDirective {
    // This snippet is parsed, but not enforced by the server, which is why we
    // don't bother adding detailed types here.
    fn definition() -> String {
        format!("directive @{AUTH_DIRECTIVE} on SCHEMA | OBJECT | FIELD_DEFINITION")
    }
}

impl<'a> Visitor<'a> for AuthDirective {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        schema_definition: &'a engine::Positioned<SchemaDefinition>,
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
        type_definition: &'a engine::Positioned<TypeDefinition>,
    ) {
        if let (Some(auth_directive), false) = (
            type_definition.node.directives.iter().find(|d| d.is_auth()),
            type_definition.node.directives.iter().any(|d| d.is_model()),
        ) {
            ctx.report_error(
                vec![auth_directive.pos],
                format!("the @{AUTH_DIRECTIVE} directive can only be used on @{MODEL_DIRECTIVE} types"),
            );
        }
    }

    // Visit fields to check that the auth directive is used correctly. Actual
    // processing happens in the model directive.
    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if let (Some(auth_directive), false) = (
            field.node.directives.iter().find(|d| d.is_auth()),
            parent_type.node.directives.iter().any(|d| d.is_model()),
        ) {
            ctx.report_error(
                vec![auth_directive.pos],
                format!("the @{AUTH_DIRECTIVE} directive can only be used on fields of @{MODEL_DIRECTIVE} types"),
            );
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use common_types::auth::Operations;
    use engine_parser::parse_schema;
    use pretty_assertions::assert_eq;
    use providers::DEFAULT_GROUPS_CLAIM;

    use super::*;
    use crate::rules::{model_directive::ModelDirective, visitor::visit};

    macro_rules! parse_test {
        ($fn_name:ident, $schema:literal, $expect:expr) => {
            parse_test!($fn_name, $schema, HashMap::new(), $expect);
        };
        ($fn_name:ident, $schema:literal, $variables:expr, $expect:expr) => {
            #[test]
            fn $fn_name() {
                let variables = $variables;
                let schema = parse_schema($schema).unwrap();
                let mut ctx = VisitorContext::new(&schema, true, &variables);
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
                let mut ctx = VisitorContext::new(&schema, true, &variables);
                visit(&mut AuthDirective, &mut ctx, &schema);
                visit(&mut ModelDirective, &mut ctx, &schema);

                assert_eq!(ctx.errors.len(), 1, "errors: {:?}", ctx.errors);
                assert_eq!(ctx.errors.get(0).unwrap().message, $err);
            }
        };
    }

    parse_test!(
        no_auth_directive,
        r"
        schema {
          query: Query
        }
        ",
        Default::default()
    );

    parse_fail!(
        anonymous_rule,
        r"
        schema @auth(
          rules: [ { allow: anonymous } ]
        ){
          query: Query
        }
        ",
        "auth rule: unknown variant `anonymous`, expected one of `private`, `public`, `groups`, `owner`"
    );

    parse_fail!(
        anonymous_rule_with_unsupported_ops,
        r"
        schema @auth(
          rules: [ { allow: anonymous, operations: [read] } ]
        ){
          query: Query
        }
        ",
        "auth rule: unknown variant `anonymous`, expected one of `private`, `public`, `groups`, `owner`"
    );

    parse_test!(
        private_rule,
        r"
        schema @auth(
          rules: [ { allow: private } ]
        ){
          query: Query
        }
        ",
        engine::AuthConfig {
            allowed_private_ops: Operations::all(),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        private_rule_with_ops,
        r"
        schema @auth(
          rules: [ { allow: private, operations: [create, delete] } ]
        ){
          query: Query
        }
        ",
        engine::AuthConfig {
            allowed_private_ops: Operations::CREATE | Operations::DELETE,
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        private_rule_with_empty_ops,
        r"
        schema @auth(
          rules: [ { allow: private, operations: [] } ]
        ){
          query: Query
        }
        ",
        engine::AuthConfig {
            allowed_private_ops: Operations::empty(),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        groups_rule,
        r#"
        schema @auth(
          rules: [ { allow: groups, groups: ["admin", "moderator"] } ],
        ){
          query: Query
        }
        "#,
        engine::AuthConfig {
            allowed_group_ops: HashMap::from_iter(vec![
                ("admin".to_string(), Operations::all()),
                ("moderator".to_string(), Operations::all()),
            ]),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        groups_rule_with_ops,
        r#"
        schema @auth(
          rules: [
            { allow: groups, groups: ["admin"] }
            { allow: groups, groups: ["moderator", "editor"], operations: ["get", "list"] }
          ],
        ){
          query: Query
        }
        "#,
        engine::AuthConfig {
            allowed_group_ops: HashMap::from_iter(vec![
                ("admin".to_string(), Operations::all()),
                ("moderator".to_string(), Operations::GET | Operations::LIST),
                ("editor".to_string(), Operations::GET | Operations::LIST)
            ]),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
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
        r"
        schema @auth(
          rules: [ { allow: groups, groups: null } ],
        ){
          query: Query
        }
        ",
        "auth rule: invalid type: null, expected a sequence"
    );

    parse_fail!(
        groups_rule_with_empty_groups,
        r"
        schema @auth(
          rules: [ { allow: groups, groups: [] } ],
        ){
          query: Query
        }
        ",
        "groups must be a non-empty list"
    );

    parse_test!(
        owner_rule,
        r"
        schema @auth(
          rules: [ { allow: owner } ],
        ){
          query: Query
        }
        ",
        engine::AuthConfig {
            allowed_owner_ops: Operations::all(),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        owner_rule_with_ops,
        r#"
        schema @auth(
          rules: [ { allow: owner, operations: ["create"] } ],
        ){
          query: Query
        }
        "#,
        engine::AuthConfig {
            allowed_owner_ops: Operations::CREATE,
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        oidc_provider,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
        ){
          query: Query
        }
        "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Oidc(engine::OidcProvider {
                issuer: "https://my.idp.com".to_string(),
                issuer_base_url: "https://my.idp.com".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_fail!(
        oidc_with_missing_field,
        r"
        schema @auth(
          providers: [ { type: oidc } ]
        ){
          query: Query
        }
        ",
        "auth provider: missing field `issuer`"
    );

    parse_fail!(
        type_auth_without_model,
        r"
        type Todo @auth(rules: []) {
          id: ID!
        }
        ",
        "the @auth directive can only be used on @model types"
    );

    parse_fail!(
        field_auth_without_model,
        r"
        type Todo {
          id: ID!
          title: String @auth(rules: [])
        }
        ",
        "the @auth directive can only be used on fields of @model types"
    );

    parse_test!(
        issuer_url_and_client_id_from_variables,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{ env.ISSUER_URL }}", clientId: "{{ env.CLIENT_ID }}" } ]
        ){
          query: Query
        }
        "#,
        HashMap::from([
            ("ISSUER_URL".to_string(), "https://my.idp.com".to_string()),
            ("CLIENT_ID".to_string(), "some-id".to_string()),
        ]),
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Oidc(engine::OidcProvider {
                issuer: "https://my.idp.com".to_string(),
                issuer_base_url: "https://my.idp.com".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: Some("some-id".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
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
        "auth provider: undefined variable `ISSUER_URL`"
    );

    parse_fail!(
        issuer_url_from_template_key_with_whitespace,
        r#"
        schema @auth(
          providers: [ { type: oidc, issuer: "{{env.ISSUER_URL   }}" } ]
        ){
          query: Query
        }
        "#,
        HashMap::new(),
        "auth provider: undefined variable `ISSUER_URL`"
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
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwt(engine::JwtProvider {
                issuer: "https://my.idp.com".to_string(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
                secret: secrecy::SecretString::new("s3cr3t".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwt_provider_with_client_id_and_issuer_url,
        r#"
        schema @auth(
          providers: [ { type: jwt, issuer: "https://my.idp.com", secret: "s3cr3t", clientId: "some-id" } ]
        ){
          query: Query
        }
        "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwt(engine::JwtProvider {
                issuer: "https://my.idp.com".to_string(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: Some("some-id".to_string()),
                secret: secrecy::SecretString::new("s3cr3t".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwt_provider_with_issuer_string,
        r#"
      schema @auth(
        providers: [ { type: jwt, issuer: "myidp", secret: "s3cr3t" } ]
      ){
        query: Query
      }
      "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwt(engine::JwtProvider {
                issuer: "myidp".to_string(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
                secret: secrecy::SecretString::new("s3cr3t".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_fail!(
        multiple_providers,
        r#"
        schema @auth(
          providers: [ { type: jwt, issuer: "myidp", secret: "s" }, { type: jwks, issuer: "https://example.com" } ]
        ){
          query: Query
        }
        "#,
        "only one auth provider can be configured right now"
    );

    parse_test!(
        oidc_provider_with_groups_claim,
        r#"
      schema @auth(
        providers: [ { type: oidc, issuer: "https://my.idp.com", groupsClaim: "grps" } ]
      ){
        query: Query
      }
      "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Oidc(engine::OidcProvider {
                issuer: "https://my.idp.com".to_string(),
                issuer_base_url: "https://my.idp.com".parse().unwrap(),
                groups_claim: "grps".to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwt_provider_with_groups_claim,
        r#"
    schema @auth(
      providers: [ { type: jwt, issuer: "myidp", secret: "s3cr3t", groupsClaim: "grps" } ]
    ){
      query: Query
    }
    "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwt(engine::JwtProvider {
                issuer: "myidp".to_string(),
                groups_claim: "grps".to_string(),
                client_id: None,
                secret: secrecy::SecretString::new("s3cr3t".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwks_provider_with_issuer,
        r#"
schema @auth(
  providers: [ { type: jwks, issuer: "http://example.com" } ]
){
  query: Query
}
"#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwks(engine::JwksProvider {
                issuer: Some("http://example.com".to_string()),
                jwks_endpoint: "http://example.com/.well-known/jwks.json".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwks_provider_with_endpoint,
        r#"
  schema @auth(
    providers: [ { type: jwks, jwksEndpoint: "http://example.com/jwks" } ]
  ){
    query: Query
  }
  "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwks(engine::JwksProvider {
                issuer: None,
                jwks_endpoint: "http://example.com/jwks".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwks_provider_with_issuer_and_endpoint,
        r#"
schema @auth(
  providers: [ { type: jwks, issuer: "myidp", jwksEndpoint: "http://example.com/jwks" } ]
){
  query: Query
}
"#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwks(engine::JwksProvider {
                issuer: Some("myidp".to_string()),
                jwks_endpoint: "http://example.com/jwks".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        jwks_provider_with_issuer_variable,
        r#"
schema @auth(
providers: [ { type: jwks, issuer: "{{ env.ISSUER_URL }}", clientId: "{{ env.CLIENT_ID }}", groupsClaim: "grps" } ]
){
query: Query
}
"#,
        HashMap::from([
            ("ISSUER_URL".to_string(), "https://my.idp.com".to_string()),
            ("CLIENT_ID".to_string(), "some-id".to_string()),
            ("GROUPS".to_string(), "grps".to_string()),
        ]),
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Jwks(engine::JwksProvider {
                issuer: Some("https://my.idp.com".to_string()),
                jwks_endpoint: "https://my.idp.com/.well-known/jwks.json".parse().unwrap(),
                groups_claim: "grps".to_string(),
                client_id: Some("some-id".to_string()),
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        oidc_provider_with_path,
        r#"
      schema @auth(
        providers: [ { type: oidc, issuer: "https://my.idp.com/some/path/" } ]
      ){
        query: Query
      }
      "#,
        engine::AuthConfig {
            provider: Some(engine::AuthProvider::Oidc(engine::OidcProvider {
                issuer: "https://my.idp.com/some/path/".to_string(),
                issuer_base_url: "https://my.idp.com/some/path/".parse().unwrap(),
                groups_claim: DEFAULT_GROUPS_CLAIM.to_string(),
                client_id: None,
            })),
            allowed_public_ops: allowed_public_ops(Operations::empty()),
            ..Default::default()
        }
    );

    parse_test!(
        public_rule,
        r"
        schema @auth(
          rules: [ { allow: public } ],
        ){
          query: Query
        }
        ",
        engine::AuthConfig {
            allowed_public_ops: Operations::all(),
            ..Default::default()
        }
    );

    parse_test!(
        public_rule_combined_with_private,
        r"
      schema @auth(
        rules: [ { allow: public, operations: [ get ] }, { allow: private } ],
      ){
        query: Query
      }
      ",
        engine::AuthConfig {
            allowed_public_ops: allowed_public_ops(Operations::GET),
            allowed_private_ops: Operations::all(),
            ..Default::default()
        }
    );

    #[cfg(feature = "local")] // Allow public introspection locally for backwards compatibility.
    pub fn allowed_public_ops(allowed_public_ops: Operations) -> Operations {
        allowed_public_ops.union(Operations::INTROSPECTION)
    }

    #[cfg(not(feature = "local"))]
    pub fn allowed_public_ops(allowed_public_ops: Operations) -> Operations {
        allowed_public_ops
    }
}
