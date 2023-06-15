use std::collections::HashMap;

use dynaql::ServerError;
use dynaql_parser::{types::ConstDirective, Positioned};
use dynaql_value::ConstValue;

use super::{operations::Operations, providers::AuthProvider, rules::AuthRule};
use crate::VisitorContext;

#[derive(Debug)]
struct InternalAuthConfig {
    allowed_private_ops: Operations,

    allowed_public_ops: Operations,

    allowed_group_ops: HashMap<String, Operations>,

    allowed_owner_ops: Operations,

    provider: Option<AuthProvider>,
}

pub fn parse_auth_config(
    ctx: &VisitorContext<'_>,
    directive: &Positioned<ConstDirective>,
    is_global: bool,
) -> Result<dynaql::AuthConfig, ServerError> {
    let value = &directive.node;
    let pos = Some(value.name.pos);

    #[derive(serde::Deserialize, Debug)]
    struct AuthDirective {
        providers: Option<Vec<AuthProvider>>,
    }

    let provider = match crate::directive_de::parse_directive::<AuthDirective>(&directive.node, ctx.variables)
        .map_err(|rule_err| {
            ServerError::new_with_locations(format!("auth provider: {}", rule_err.message), rule_err.locations)
        })?
        .providers
    {
        None => Ok(None),
        Some(single) if single.len() == 1 => single.into_iter().next().unwrap().validate().map(Some),
        Some(empty) if empty.is_empty() => Err(ServerError::new("auth providers must be a non-empty list", pos)),
        Some(_) => Err(ServerError::new(
            "only one auth provider can be configured right now",
            pos,
        )),
    }?;

    // XXX: introduce a separate type for non-global directives if we need more custom behavior
    if !is_global && provider.is_some() {
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

    let allowed_public_ops: Operations = rules
        .iter()
        .filter_map(|rule| match rule {
            AuthRule::Public { operations, .. } => Some(operations.values().clone()),
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
        allowed_public_ops,
        allowed_group_ops,
        allowed_owner_ops,
        provider,
    }))
}

impl From<InternalAuthConfig> for dynaql::AuthConfig {
    fn from(auth: InternalAuthConfig) -> Self {
        Self {
            allowed_private_ops: auth.allowed_private_ops.into(),

            allowed_public_ops: auth.allowed_public_ops.into(),

            allowed_group_ops: auth
                .allowed_group_ops
                .into_iter()
                .map(|(group, ops)| (group, ops.into()))
                .collect(),

            allowed_owner_ops: auth.allowed_owner_ops.into(),

            provider: match auth.provider {
                Some(AuthProvider::Oidc {
                    issuer,
                    groups_claim,
                    client_id,
                }) => {
                    let issuer_base_url = issuer.parse().expect("issuer format must have been validated");
                    Some(dynaql::AuthProvider::Oidc(dynaql::OidcProvider {
                        issuer,
                        issuer_base_url,
                        groups_claim,
                        client_id,
                    }))
                }
                Some(AuthProvider::Jwks {
                    issuer,
                    jwks_endpoint,
                    groups_claim,
                    client_id,
                }) => {
                    let jwks_endpoint = jwks_endpoint.as_ref().expect("must have been set");
                    let jwks_endpoint = jwks_endpoint.parse::<url::Url>().expect("must be a valid URL");
                    Some(dynaql::AuthProvider::Jwks(dynaql::JwksProvider {
                        jwks_endpoint,
                        issuer,
                        groups_claim,
                        client_id,
                    }))
                }
                Some(AuthProvider::Jwt {
                    issuer,
                    groups_claim,
                    client_id,
                    secret,
                }) => Some(dynaql::AuthProvider::Jwt(dynaql::JwtProvider {
                    issuer,
                    groups_claim,
                    client_id,
                    secret: secrecy::SecretString::new(secret),
                })),
                None => None,
            },
        }
    }
}
