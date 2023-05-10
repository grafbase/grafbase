use std::collections::HashMap;

use dynaql::ServerError;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

use super::{operations::Operations, providers::AuthProvider, rules::AuthRule};
use crate::VisitorContext;

const ENV_VAR_ERROR: &str = "environment variables have been expanded by now";

#[derive(Debug)]
struct InternalAuthConfig {
    allowed_private_ops: Operations,

    allowed_group_ops: HashMap<String, Operations>,

    allowed_owner_ops: Operations,

    providers: Vec<AuthProvider>,
}

pub fn parse_auth_config(
    ctx: &VisitorContext<'_>,
    value: &ConstDirective,
    is_global: bool,
) -> Result<dynaql::AuthConfig, ServerError> {
    let pos = Some(value.name.pos);

    let providers = match value.get_argument("providers") {
        Some(arg) => match &arg.node {
            ConstValue::List(value) if value.len() > 1 => {
                return Err(ServerError::new(
                    "only one auth provider can be configured right now",
                    pos,
                ))
            }
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
                .map(AuthRule::from_value)
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

    Ok(dynaql::AuthConfig::from(InternalAuthConfig {
        allowed_private_ops,
        allowed_group_ops,
        allowed_owner_ops,
        providers,
    }))
}

impl From<InternalAuthConfig> for dynaql::AuthConfig {
    fn from(auth: InternalAuthConfig) -> Self {
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
                .filter_map(|provider| match provider {
                    AuthProvider::Oidc {
                        issuer,
                        groups_claim,
                        client_id,
                    } => {
                        let issuer: String = issuer.as_fully_evaluated_str().expect(ENV_VAR_ERROR).parse().unwrap();
                        let issuer_base_url = issuer.parse().expect("issuer format must have been validated");
                        Some(dynaql::OidcProvider {
                            issuer,
                            issuer_base_url,
                            groups_claim: groups_claim.clone(),
                            client_id: client_id
                                .as_ref()
                                .map(|id| id.as_fully_evaluated_str().expect(ENV_VAR_ERROR).to_string()),
                        })
                    }
                    _ => None,
                })
                .collect(),

            jwks_providers: auth
                .providers
                .iter()
                .filter_map(|provider| match provider {
                    AuthProvider::Jwks {
                        issuer,
                        jwks_endpoint,
                        groups_claim,
                        client_id,
                    } => {
                        let jwks_endpoint = jwks_endpoint
                            .as_ref()
                            .expect("must have been set")
                            .as_fully_evaluated_str()
                            .expect("must be evaluated");
                        let jwks_endpoint = jwks_endpoint.parse::<url::Url>().expect("must be a valid URL");
                        let issuer = issuer
                            .as_ref()
                            .map(|issuer| issuer.as_fully_evaluated_str().expect(ENV_VAR_ERROR).to_string());
                        Some(dynaql::JwksProvider {
                            jwks_endpoint,
                            issuer,
                            groups_claim: groups_claim.clone(),
                            client_id: client_id
                                .as_ref()
                                .map(|id| id.as_fully_evaluated_str().expect(ENV_VAR_ERROR).to_string()),
                        })
                    }
                    _ => None,
                })
                .collect(),

            jwt_providers: auth
                .providers
                .iter()
                .filter_map(|provider| match provider {
                    AuthProvider::Jwt {
                        issuer,
                        groups_claim,
                        client_id,
                        secret,
                    } => Some(dynaql::JwtProvider {
                        issuer: issuer.as_fully_evaluated_str().expect(ENV_VAR_ERROR).parse().unwrap(),
                        groups_claim: groups_claim.clone(),
                        client_id: client_id
                            .as_ref()
                            .map(|id| id.as_fully_evaluated_str().expect(ENV_VAR_ERROR).to_string()),
                        secret: secrecy::SecretString::new(
                            secret.as_fully_evaluated_str().expect(ENV_VAR_ERROR).to_string(),
                        ),
                    }),
                    _ => None,
                })
                .collect(),
        }
    }
}
