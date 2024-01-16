use std::{collections::BTreeMap, sync::Arc};

use common_types::{auth::ExecutionAuth, UdfKind};
use engine::{AuthConfig, AuthProvider, AuthorizerProvider};
use futures_util::TryFutureExt;
use jwt_verifier::{VerificationError, VerifiedToken};
use runtime::{
    kv::KvStore,
    udf::{AuthorizerRequestPayload, CustomResolverResponse, UdfInvoker},
};

use super::RequestContext;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum AuthError {
    #[error("verification error: {0}")]
    VerificationError(#[from] VerificationError),
    #[error("authorizer invocation error")]
    UdfError,
    #[error("authorizer returned invalid token claims: {0}")]
    InvalidTokenClaims(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AdminAuthError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[async_trait::async_trait]
pub trait Authorizer: Send + Sync {
    type Context;
    async fn authorize_admin_request(
        &self,
        ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError>;

    async fn authorize_request(
        &self,
        ctx: &Arc<Self::Context>,
        _request: &engine::Request,
    ) -> Result<ExecutionAuth, AuthError>;
}

pub fn build_token_based_auth(verified_token: VerifiedToken, auth_config: &AuthConfig) -> ExecutionAuth {
    // Get the global level group and owner based operations that are allowed.
    let private_public_and_group_ops = auth_config.private_public_and_group_based_ops(&verified_token.groups);
    let allowed_owner_ops = auth_config.owner_based_ops();

    // It's fine for ops and groups to be empty as there might
    // be some model-level auth rule evaluated later
    let subject_and_owner_ops = verified_token.identity.and_then(|subject| {
        // Turn off owner-based auth if no operations are allowed.
        if allowed_owner_ops.is_empty() {
            None
        } else {
            Some((subject, allowed_owner_ops))
        }
    });
    ExecutionAuth::new_from_token(
        private_public_and_group_ops,
        verified_token.groups,
        subject_and_owner_ops,
        verified_token.token_claims,
    )
}

pub fn build_public_auth(auth_config: &AuthConfig) -> ExecutionAuth {
    ExecutionAuth::Public {
        global_ops: auth_config.allowed_public_ops,
    }
}

#[allow(clippy::panic)]
pub async fn authorize_request(
    jwks_cache: Option<KvStore>,
    auth_invoker: &impl UdfInvoker<AuthorizerRequestPayload>,
    auth_config: &AuthConfig,
    ctx: &impl RequestContext,
    // Should be retrieved from both header AND query param for backwards compatibility...
    authorization_header: Option<String>,
) -> Result<ExecutionAuth, AuthError> {
    let id_token = authorization_header.and_then(|val| val.strip_prefix("Bearer ").map(str::to_string));
    let result = match (id_token, &auth_config.provider) {
        // API key has precedence over ID token
        (Some(token), Some(AuthProvider::Oidc(oidc_provider))) => {
            let client = jwt_verifier::Client {
                trace_id: ctx.ray_id(),
                jwks_cache,
                groups_claim: Some(&oidc_provider.groups_claim),
                client_id: oidc_provider.client_id.as_deref(),
                time_opts: Default::default(),
                http_client: Default::default(),
            };
            client
                .verify_rs_token_using_oidc_discovery(&token, &oidc_provider.issuer_base_url, &oidc_provider.issuer)
                .inspect_err(|err| log::warn!(ctx.ray_id(), "Unauthorized: {err:?}"))
                .await
                .map(|verified_token| build_token_based_auth(verified_token, auth_config))?
        }
        (Some(token), Some(AuthProvider::Jwks(jwks_provider))) => {
            let client = jwt_verifier::Client {
                trace_id: ctx.ray_id(),
                jwks_cache,
                groups_claim: Some(&jwks_provider.groups_claim),
                client_id: jwks_provider.client_id.as_deref(),
                time_opts: Default::default(),
                http_client: Default::default(),
            };

            client
                .verify_rs_token_using_jwks_endpoint(
                    &token,
                    &jwks_provider.jwks_endpoint,
                    jwks_provider.issuer.as_deref(),
                )
                .inspect_err(|err| log::warn!(ctx.ray_id(), "Unauthorized: {err:?}"))
                .await
                .map(|verified_token| build_token_based_auth(verified_token, auth_config))?
        }
        (Some(token), Some(AuthProvider::Jwt(jwt_provider))) => {
            let client = jwt_verifier::Client {
                trace_id: ctx.ray_id(),
                groups_claim: Some(&jwt_provider.groups_claim),
                client_id: jwt_provider.client_id.as_deref(),
                jwks_cache,
                time_opts: Default::default(),
                http_client: Default::default(),
            };

            client
                .verify_hs_token(token, &jwt_provider.issuer, &jwt_provider.secret)
                .map_err(|err| {
                    log::warn!(ctx.ray_id(), "Unauthorized: {err:?}");
                    err
                })
                .map(|verified_token| build_token_based_auth(verified_token, auth_config))?
        }
        (_, Some(AuthProvider::Authorizer(AuthorizerProvider { name }))) => {
            call_authorizer(ctx, name.clone(), auth_invoker, auth_config).await?
        }
        _ => build_public_auth(auth_config),
    };
    log::debug!(
        ctx.ray_id(),
        "Authorizing request using {auth_config:?} produces {result:?}"
    );
    Ok(result)
}

async fn call_authorizer(
    ctx: &impl RequestContext,
    name: String,
    invoker: &impl UdfInvoker<AuthorizerRequestPayload>,
    auth_config: &AuthConfig,
) -> Result<ExecutionAuth, AuthError> {
    let ray_id = ctx.ray_id();
    let request = runtime::udf::UdfRequest {
        name: &name,
        request_id: ray_id,
        udf_kind: UdfKind::Authorizer,
        payload: runtime::udf::AuthorizerRequestPayload {
            context: runtime::udf::UdfRequestContext {
                request: runtime::udf::UdfRequestContextRequest {
                    headers: serde_json::to_value(ctx.headers_as_map()).expect("must be valid"),
                },
            },
        },
    };
    let value = invoker
        .invoke(ray_id, request)
        .map_err(|err| {
            log::warn!(ray_id, "authorizer failed: {err:?}");
            AuthError::VerificationError(VerificationError::Authorizer)
        })
        .and_then(|response| {
            futures_util::future::ready(match response {
                CustomResolverResponse::Success(value) => Ok(value),
                CustomResolverResponse::GraphQLError { .. } | CustomResolverResponse::Error(_) => {
                    Err(AuthError::UdfError)
                }
            })
        })
        .await?;

    log::trace!(ray_id, "Authorizer response: {value:?}");
    if let Some(identity) = value.get("identity") {
        let sub = match identity.get("sub") {
            Some(serde_json::Value::String(sub)) => Ok(Some(sub.clone())),
            None => Ok(None),
            other => {
                log::warn!(
                    ray_id,
                    "authorizer contract violation while getting subject, expected string, got {other:?}"
                );
                Err(AuthError::VerificationError(VerificationError::Authorizer))
            }
        }?;
        let groups = match identity.get("groups") {
            Some(serde_json::Value::Array(groups)) => groups
                .iter()
                .map(|val| match val {
                    serde_json::Value::String(val) => Ok(val.clone()),
                    other => {
                        log::warn!(
                            ray_id,
                            "authorizer contract violation while getting groups, expected string, got {other:?}"
                        );
                        Err(AuthError::VerificationError(VerificationError::Authorizer))
                    }
                })
                .collect::<Result<_, _>>(),
            None => Ok(Default::default()),
            other => {
                log::warn!(
                    ray_id,
                    "authorizer contract violation while getting groups, expected array of strings, got {other:?}"
                );
                Err(AuthError::VerificationError(VerificationError::Authorizer))
            }
        }?;

        let token_claims = serde_json::from_value::<BTreeMap<String, serde_json::Value>>(identity.clone())
            .map_err(|_| AuthError::InvalidTokenClaims(identity.to_string()))?;

        let verified_token = VerifiedToken {
            identity: sub,
            groups,
            token_claims,
        };
        log::debug!(ray_id, "Authorizer verified {verified_token:?}");
        Ok(build_token_based_auth(verified_token, auth_config))
    } else {
        // no identity returned, public access.
        Ok(build_public_auth(auth_config))
    }
}
